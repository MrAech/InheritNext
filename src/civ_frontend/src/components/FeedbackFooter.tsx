import React from "react";

const FeedbackFooter: React.FC = () => (
  <footer className="fixed bottom-0 left-0 w-full bg-card text-center py-2 z-40 shadow">
    <span className="text-sm text-muted-foreground">
      Evaluator feedback?&nbsp;
      <a
        href="https://github.com/your-repo/issues"
        target="_blank"
        rel="noopener noreferrer"
        className="underline text-primary"
      >
        Submit on GitHub
      </a>
    </span>
  </footer>
);

export default FeedbackFooter;