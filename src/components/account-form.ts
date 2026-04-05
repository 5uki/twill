import type {
  AccountConnectionCommandInput,
  AddAccountCommandInput,
  MailSecurity,
} from "../lib/app-types";

export interface AccountFormDraft {
  displayName: string;
  email: string;
  login: string;
  password: string;
  imapHost: string;
  imapPort: string;
  imapSecurity: MailSecurity;
  smtpHost: string;
  smtpPort: string;
  smtpSecurity: MailSecurity;
}

export function createEmptyAccountFormDraft(): AccountFormDraft {
  return {
    displayName: "",
    email: "",
    login: "",
    password: "",
    imapHost: "",
    imapPort: "993",
    imapSecurity: "tls",
    smtpHost: "",
    smtpPort: "587",
    smtpSecurity: "start_tls",
  };
}

export function buildAccountCommandInput(
  draft: AccountFormDraft,
): AddAccountCommandInput {
  return {
    ...buildBaseCommandInput(draft),
    password: draft.password.trim(),
  };
}

export function buildAccountConnectionCommandInput(
  draft: AccountFormDraft,
): AccountConnectionCommandInput {
  return buildBaseCommandInput(draft);
}

function buildBaseCommandInput(
  draft: AccountFormDraft,
): Omit<AddAccountCommandInput, "password"> {
  const email = draft.email.trim();
  const login = draft.login.trim() || email;

  return {
    display_name: draft.displayName.trim(),
    email,
    login,
    imap: {
      host: draft.imapHost.trim(),
      port: parsePort(draft.imapPort, "IMAP"),
      security: draft.imapSecurity,
    },
    smtp: {
      host: draft.smtpHost.trim(),
      port: parsePort(draft.smtpPort, "SMTP"),
      security: draft.smtpSecurity,
    },
  };
}

function parsePort(value: string, label: string): number {
  const port = Number.parseInt(value.trim(), 10);

  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error(`${label} 端口必须是 1 到 65535 之间的整数`);
  }

  return port;
}
