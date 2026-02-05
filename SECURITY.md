# Security Policy

## Reporting a Vulnerability

We take the security of Cosmian VM seriously. If you discover a security vulnerability, please report it responsibly by following these steps:

### Private Reporting

Please **do not** report security vulnerabilities through public GitHub issues. Instead, please use one of the following methods:

1. **GitHub Security Advisories** (Preferred): Use the [private vulnerability reporting feature](https://github.com/Cosmian/cosmian_vm/security/advisories/new) on GitHub
2. **Email**: Send details to [tech@cosmian.com](mailto:tech@cosmian.com)

### What to Include

When reporting a vulnerability, please include as much of the following information as possible:

- A clear description of the vulnerability
- Steps to reproduce the issue
- Potential impact of the vulnerability
- Suggested fix (if you have one)
- Your contact information

### Response Timeline

- **Initial Response**: We will acknowledge receipt of your vulnerability report within 48 hours
- **Investigation**: We will investigate and validate the vulnerability within 5 business days
- **Fix Development**: We will work to develop and test a fix as quickly as possible
- **Disclosure**: We will coordinate the disclosure timeline with you

## Supported Versions

The following versions of Cosmian VM are currently supported with security updates:

| Version | Supported         |
| ------- | ----------------- |
| 1.3.x   | ✅ Yes             |
| 1.2.x   | ⚠️ Limited Support |
| < 1.2   | ❌ No              |

**Note**: Base images are also regularly updated. Please refer to our [base images changelog](CHANGELOG_BASE_IMAGES.md) for security updates to the underlying OS images.

## Known Security Advisories

The following table lists security advisories that are currently being tracked or have been assessed for this project:

| ID                | Description                                              | Status  | Reason                                                   |
| ----------------- | -------------------------------------------------------- | ------- | -------------------------------------------------------- |
| RUSTSEC-2020-0071 | Potential segfault in time crate                         | Ignored | Dependency via certtool->acme-lib - limited impact       |
| RUSTSEC-2023-0071 | RSA crate vulnerability affecting signature verification | Ignored | Under evaluation - specific use case may not be affected |
| RUSTSEC-2021-0145 | atty unmaintained                                        | Ignored | Waiting for tdx-attest-rs to be upgraded                 |
| RUSTSEC-2021-0139 | ansi_term unmaintained                                   | Ignored | Waiting for tdx-attest-rs to be upgraded                 |
| RUSTSEC-2024-0375 | atty unmaintained                                        | Ignored | Low priority - related to terminal handling              |

### Advisory Details

These security advisories are tracked in our `deny.toml` configuration file and are regularly reviewed by our security team. Most ignored advisories are due to:

1. **Transitive dependencies**: Issues in dependencies of dependencies that we cannot directly control
2. **Limited impact**: Vulnerabilities that don't affect the core security model of Cosmian VM
3. **Waiting for upstream fixes**: Issues that require updates from upstream maintainers

## Security Architecture

Cosmian VM is built around a comprehensive security model that includes:

### Trusted Execution Environment (TEE)

- **AMD SEV-SNP**: Secure Encrypted Virtualization with Secure Nested Paging
- **Intel TDX**: Trust Domain Extensions for memory encryption
- Hardware-based memory encryption protecting against host-level attacks

### Trusted Platform Module (TPM)

- **TPM 2.0 / vTPM**: Secure storage of cryptographic keys and measurements
- **Attestation**: Remote verification of platform integrity
- **Secret storage**: Secure key management for LUKS containers

### Integrity Measurement Architecture (IMA)

- **Runtime measurement**: Continuous monitoring of file integrity
- **Measurement log**: Cryptographic record of all executed binaries
- **Baseline comparison**: Verification against known-good snapshots

## Security Best Practices

When deploying Cosmian VM, we recommend:

### Infrastructure Security

1. **Network isolation**: Deploy VMs in private networks with appropriate firewall rules
2. **Certificate management**: Use valid TLS certificates (Let's Encrypt or enterprise CA)
3. **Access control**: Implement strict SSH key management and disable password authentication
4. **Monitoring**: Enable logging and monitoring for security events

### Operational Security

1. **Snapshot verification**: Always verify snapshots before production deployment
2. **Regular snapshot verification**: Periodically re-verify IMA measurements against baselines

### TEE-Specific Considerations

1. **Attestation verification**: Always verify TEE attestations before trusting a VM
2. **Hardware validation**: Ensure cloud instances support required TEE features
3. **Measurement validation**: Regularly compare IMA measurements against baselines
4. **Encrypted storage**: Use TPM-backed LUKS encryption for sensitive data

## Confidential Computing Threat Model

Cosmian VM is designed to protect against:

- **Honest-but-curious cloud providers**: Memory encryption prevents unauthorized access
- **Malicious cloud infrastructure**: TEE attestation validates platform integrity
- **Compromised hypervisors**: Hardware-based isolation provides protection
- **Memory attacks**: Encrypted memory prevents direct access to sensitive data

### Out of Scope

- **Application-level vulnerabilities**: Secure coding practices still required
- **Side-channel attacks**: Some advanced attacks may still be possible
- **Physical access**: Direct hardware access by attackers is not protected

## Contact

For general security questions or concerns, please contact us at [tech@cosmian.com](mailto:tech@cosmian.com).

For immediate security issues, please use the private reporting methods described above.

## Security Resources

- [TEE Attestation Documentation](https://github.com/Cosmian/tee-tools)
- [TPM 2.0 Specification](https://trustedcomputinggroup.org/resource/tpm-library-specification/)
- [AMD SEV-SNP Documentation](https://developer.amd.com/sev/)
- [Intel TDX Documentation](https://www.intel.com/content/www/us/en/developer/tools/trust-domain-extensions/overview.html)
- [Confidential Computing Consortium](https://confidentialcomputing.io/)
