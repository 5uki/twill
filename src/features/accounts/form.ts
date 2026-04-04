import type { AddAccountInput, AccountConnectionTestInput, MailSecurity } from "./model";

export interface AccountFormState {
  displayName: string;
  email: string;
  login: string;
  imapHost: string;
  imapPort: string;
  imapSecurity: MailSecurity;
  smtpHost: string;
  smtpPort: string;
  smtpSecurity: MailSecurity;
}

export function createDefaultAccountFormState(): AccountFormState {
  return {
    displayName: "",
    email: "",
    login: "",
    imapHost: "imap.example.com",
    imapPort: "993",
    imapSecurity: "tls",
    smtpHost: "smtp.example.com",
    smtpPort: "587",
    smtpSecurity: "start_tls",
  };
}

export function toAddAccountInput(form: AccountFormState): AddAccountInput {
  return {
    display_name: form.displayName.trim(),
    email: form.email.trim(),
    login: form.login.trim(),
    imap: {
      host: form.imapHost.trim(),
      port: parsePort(form.imapPort),
      security: form.imapSecurity,
    },
    smtp: {
      host: form.smtpHost.trim(),
      port: parsePort(form.smtpPort),
      security: form.smtpSecurity,
    },
  };
}

export function toConnectionTestInput(form: AccountFormState): AccountConnectionTestInput {
  return {
    display_name: form.displayName.trim(),
    email: form.email.trim(),
    login: form.login.trim(),
    imap: {
      host: form.imapHost.trim(),
      port: parsePort(form.imapPort),
      security: form.imapSecurity,
    },
    smtp: {
      host: form.smtpHost.trim(),
      port: parsePort(form.smtpPort),
      security: form.smtpSecurity,
    },
  };
}

function parsePort(value: string) {
  const parsed = Number.parseInt(value.trim(), 10);

  return Number.isFinite(parsed) ? parsed : 0;
}
