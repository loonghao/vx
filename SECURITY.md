# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.9.x   | :white_check_mark: |
| < 0.9.0 | :x:                |

## Reporting a Vulnerability

We take the security of vx seriously. If you believe you've found a security vulnerability, please report it to us as described below.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to **hal.long@outlook.com** with the subject line **"vx Security Vulnerability"**.

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

## What to Include

To help us understand and resolve the issue quickly, please include:

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

## Disclosure Policy

- We will acknowledge receipt of your vulnerability report within 48 hours.
- We will provide an estimated time frame for a fix.
- We will notify you when the vulnerability is fixed.
- We will publicly disclose the vulnerability after the fix has been released.

## Security Best Practices

- **Do not run `sudo vx install`** — vx manages user-level installations in `~/.vx/`.
- Downloads are from official sources (GitHub Releases, official APIs); checksums are verified automatically.
- The `permissions` field in `provider.star` declares which network hosts a provider may access.
- Set `GITHUB_TOKEN` to avoid GitHub API rate limits.
- Use `vx doctor` to diagnose environment security issues.

## Responsible Disclosure

We encourage responsible disclosure of security vulnerabilities. We will not take legal action against researchers who:

- Share vulnerability details with us first
- Allow us reasonable time to fix the issue before public disclosure
- Do not exploit the vulnerability or cause harm

Thank you for helping keep vx and its users safe!
