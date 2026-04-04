import { describe, expect, test } from "bun:test";
import {
  createDefaultAccountFormState,
  toAddAccountInput,
  toConnectionTestInput,
} from "../../src/features/accounts/form";

describe("account form helpers", () => {
  test("creates a default onboarding form with common secure ports", () => {
    const form = createDefaultAccountFormState();

    expect(form.imapPort).toBe("993");
    expect(form.imapSecurity).toBe("tls");
    expect(form.smtpPort).toBe("587");
    expect(form.smtpSecurity).toBe("start_tls");
  });

  test("trims fields and converts ports for add account payload", () => {
    const payload = toAddAccountInput({
      displayName: "  Primary Gmail  ",
      email: "  primary@example.com  ",
      login: "  primary@example.com  ",
      imapHost: "  imap.example.com  ",
      imapPort: " 993 ",
      imapSecurity: "tls",
      smtpHost: "  smtp.example.com  ",
      smtpPort: " 587 ",
      smtpSecurity: "start_tls",
    });

    expect(payload).toEqual({
      display_name: "Primary Gmail",
      email: "primary@example.com",
      login: "primary@example.com",
      imap: {
        host: "imap.example.com",
        port: 993,
        security: "tls",
      },
      smtp: {
        host: "smtp.example.com",
        port: 587,
        security: "start_tls",
      },
    });
  });

  test("falls back to zero when a port cannot be parsed", () => {
    const payload = toConnectionTestInput({
      displayName: "Primary Gmail",
      email: "primary@example.com",
      login: "primary@example.com",
      imapHost: "imap.example.com",
      imapPort: "bad-port",
      imapSecurity: "tls",
      smtpHost: "smtp.example.com",
      smtpPort: "",
      smtpSecurity: "start_tls",
    });

    expect(payload.imap.port).toBe(0);
    expect(payload.smtp.port).toBe(0);
  });
});
