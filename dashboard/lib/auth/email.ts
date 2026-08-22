import nodemailer from 'nodemailer';

export interface EmailSendResult {
  sent: boolean;
  provider: 'smtp' | 'resend' | 'console';
  message: string;
}

/**
 * Sends real OTP verification email via SMTP or Resend API
 */
export async function sendVerificationEmail(
  toEmail: string,
  code: string
): Promise<EmailSendResult> {
  const subject = `Your DAGR Hypervisor Login Code: ${code}`;
  const htmlContent = `
    <div style="background-color: #000000; color: #E4E4E7; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 40px 20px; border-radius: 12px; max-width: 540px; margin: 0 auto; border: 1px solid rgba(255, 255, 255, 0.1);">
      <div style="margin-bottom: 24px; text-align: center;">
        <h1 style="font-size: 24px; font-weight: 800; letter-spacing: -0.04em; margin: 0; color: #FFFFFF; text-transform: lowercase;">dagr</h1>
        <p style="font-size: 12px; color: #71717A; font-family: monospace; margin-top: 4px;">Zero-Trust AST Slicing & FinOps Telemetry Plane</p>
      </div>
      
      <div style="background: rgba(18, 18, 22, 0.9); border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 12px; padding: 24px; text-align: center;">
        <p style="font-size: 14px; color: #A1A1AA; margin: 0 0 16px 0;">Use the following single-use verification code to sign in to your organization workspace:</p>
        
        <div style="background: #000000; border: 1px solid rgba(255, 255, 255, 0.2); border-radius: 8px; padding: 16px; font-size: 32px; font-weight: 800; font-family: monospace; letter-spacing: 0.25em; color: #FFFFFF; display: inline-block; margin: 8px 0;">
          ${code}
        </div>
        
        <p style="font-size: 12px; color: #71717A; font-family: monospace; margin: 16px 0 0 0;">This code expires in 15 minutes. Never share this code with anyone.</p>
      </div>
      
      <div style="margin-top: 24px; text-align: center; font-size: 11px; color: #52525B; font-family: monospace;">
        DAGR Native Hypervisor Engine • Zero-PII • Deterministic Program Slicing
      </div>
    </div>
  `;

  // 1. Check for Resend API Key
  const resendApiKey = process.env.RESEND_API_KEY;
  if (resendApiKey) {
    try {
      const fromEmail = process.env.EMAIL_FROM || 'DAGR Auth <auth@dagr.dev>';
      const res = await fetch('https://api.resend.com/emails', {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${resendApiKey}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          from: fromEmail,
          to: toEmail,
          subject,
          html: htmlContent,
        }),
      });

      if (res.ok) {
        return {
          sent: true,
          provider: 'resend',
          message: `Verification code dispatched via Resend to ${toEmail}`,
        };
      }
    } catch (e: any) {
      console.error('[EMAIL ERROR] Resend dispatch failed:', e.message);
    }
  }

  // 2. Check for SMTP Configuration (Gmail, Outlook, SendGrid, Amazon SES, or custom SMTP)
  const smtpHost = process.env.SMTP_HOST;
  const smtpUser = process.env.SMTP_USER;
  const smtpPass = process.env.SMTP_PASS;

  if (smtpHost && smtpUser && smtpPass) {
    try {
      const transporter = nodemailer.createTransport({
        host: smtpHost,
        port: parseInt(process.env.SMTP_PORT || '587', 10),
        secure: process.env.SMTP_SECURE === 'true',
        auth: {
          user: smtpUser,
          pass: smtpPass,
        },
      });

      await transporter.sendMail({
        from: process.env.EMAIL_FROM || `"DAGR Auth" <${smtpUser}>`,
        to: toEmail,
        subject,
        html: htmlContent,
      });

      return {
        sent: true,
        provider: 'smtp',
        message: `Verification code sent via SMTP server (${smtpHost}) to ${toEmail}`,
      };
    } catch (e: any) {
      console.error('[EMAIL ERROR] SMTP dispatch failed:', e.message);
    }
  }

  // 3. Fallback: Log clearly in server console
  console.log(
    `\n\x1b[1;33m[DAGR AUTH]\x1b[0m Verification code for \x1b[1;36m${toEmail}\x1b[0m: \x1b[1;32m${code}\x1b[0m\n` +
      `  (To deliver actual emails to inboxes, configure SMTP_HOST/SMTP_USER/SMTP_PASS or RESEND_API_KEY in .env.local)\n`
  );

  return {
    sent: false,
    provider: 'console',
    message: `Verification code generated: ${code}`,
  };
}
