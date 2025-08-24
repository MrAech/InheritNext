use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;

// Simple delivery result to allow adapters to signal retry vs terminal failure vs success.
pub enum DeliveryResult {
    Sent,
    RetryLater { reason: String, backoff_secs: u64 },
    Failed { reason: String },
}

// Notification adapter trait: implement per channel.
pub trait NotificationAdapter {
    fn send(&self, channel: &NotificationChannel, template: &str, payload: &str) -> DeliveryResult;
}

// Basic log adapter (placeholder): always succeeds immediately. Could simulate transient errors based on template prefix (e.g., "fail:" or "retry:").
pub struct LogAdapter;
impl NotificationAdapter for LogAdapter {
    fn send(
        &self,
        _channel: &NotificationChannel,
        template: &str,
        _payload: &str,
    ) -> DeliveryResult {
        if let Some(rest) = template.strip_prefix("retry:") {
            return DeliveryResult::RetryLater {
                reason: format!("simulated_retry: {}", rest),
                backoff_secs: 30,
            };
        }
        if let Some(rest) = template.strip_prefix("fail:") {
            return DeliveryResult::Failed {
                reason: format!("simulated_fail: {}", rest),
            };
        }
        DeliveryResult::Sent
    }
}

fn adapter_for(_channel: &NotificationChannel) -> &'static dyn NotificationAdapter {
    &LogAdapter
}

// Queue a notification for the calling user (owner-centric for now). Returns notification id.
pub fn enqueue_notification(
    channel: NotificationChannel,
    template: String,
    payload: String,
) -> Result<u64, CivError> {
    let caller = crate::api::common::user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            let next_id = u.notifications.iter().map(|n| n.id).max().unwrap_or(0) + 1;

            let rec = NotificationRecord {
                id: next_id,
                channel: channel.clone(),
                template: template.clone(),
                payload,
                queued_at: now_secs(),
                sent_at: None,
                success: None,
                attempts: 0,
            };

            u.notifications.push(rec);

            // borrow ended, now safe to use u again
            push_audit(
                u,
                AuditEventKind::NotificationQueued {
                    channel: format!("{:?}", channel),
                    template,
                },
            );

            Ok(next_id)
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Process up to max notifications for a specific user (called by maintenance). Simulated send.
pub fn process_notifications_for(user: &str, max: usize) {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(user) {
            let now = now_secs();
            let mut processed = 0usize;

            // collect changes and audit events first
            let mut audits: Vec<AuditEventKind> = Vec::new();

            for rec in u.notifications.iter_mut().filter(|r| r.sent_at.is_none()) {
                if processed >= max {
                    break;
                }
                rec.attempts = rec.attempts.saturating_add(1);
                let adapter = adapter_for(&rec.channel);
                match adapter.send(&rec.channel, &rec.template, &rec.payload) {
                    DeliveryResult::Sent => {
                        rec.sent_at = Some(now);
                        rec.success = Some(true);
                        processed += 1;
                        audits.push(AuditEventKind::NotificationSent {
                            channel: format!("{:?}", rec.channel.clone()),
                            template: rec.template.clone(),
                            success: true,
                        });
                    }
                    DeliveryResult::RetryLater {
                        reason,
                        backoff_secs,
                    } => {
                        // leave sent_at None so it will be retried; optionally store reason in success=false to allow UI
                        rec.success = None; // still pending
                                            // simple backoff: push an audit to trace
                        audits.push(AuditEventKind::NotificationSent {
                            channel: format!("{:?}", rec.channel.clone()),
                            template: format!("{} (retry scheduled {})", rec.template, reason),
                            success: false,
                        });
                        // we could store a per-record next_attempt_after if desired (add field). For now rely on attempts count gating externally.
                    }
                    DeliveryResult::Failed { reason } => {
                        rec.sent_at = Some(now);
                        rec.success = Some(false);
                        processed += 1;
                        audits.push(AuditEventKind::NotificationSent {
                            channel: format!("{:?}", rec.channel.clone()),
                            template: format!("{} (failed {})", rec.template, reason),
                            success: false,
                        });
                    }
                }
            }

            // borrow on rec ended, now replay audits on u
            for event in audits {
                push_audit(u, event);
            }
        }
    });
}
