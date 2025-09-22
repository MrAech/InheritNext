export async function pbkdf2Hex(
  input: string,
  salt: string,
  iterations = 100_000,
  keyLen = 32,
): Promise<string> {
  const enc = new TextEncoder();
  const pass = enc.encode(input);
  const s = enc.encode(salt);
  const key = await crypto.subtle.importKey(
    "raw",
    pass,
    { name: "PBKDF2" },
    false,
    ["deriveBits"],
  );
  const derived = await crypto.subtle.deriveBits(
    { name: "PBKDF2", salt: s, iterations, hash: "SHA-256" },
    key,
    keyLen * 8,
  );
  const bytes = new Uint8Array(derived);
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

export async function sha256Hex(input: string): Promise<string> {
  const enc = new TextEncoder();
  const data = enc.encode(input);
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  const bytes = new Uint8Array(hashBuffer);
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
