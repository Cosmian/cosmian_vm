# Cosmian VM - Copilot Instructions

## Project Overview

Cosmian VM is a confidential computing solution that creates Linux-based system images for Trusted Execution Environments (TEE). The project focuses on providing verifiable and secure virtual machines with encrypted memory using AMD SEV-SNP and Intel TDX technologies.

## Repository Structure

This is a Rust workspace with the following key components:

- **`crate/agent`** - VM agent running inside confidential VMs to handle attestations and collaterals
- **`crate/cli`** - Client CLI tool (`cosmian_vm`) for interacting with and verifying remote instances
- **`crate/client`** - Shared client library for VM communication
- **`crate/certtool`** - Certificate generation tool (`cosmian_certtool`) for Let's Encrypt and RATLS certificates
- **`crate/ima`** - Integrity Measurement Architecture support for file integrity verification
- **`ansible/`** - Deployment playbooks for cloud providers (AWS, Azure, GCP)
- **`packer/`** - Configuration for building base images
- **`pkg/`** - Packaging configurations for Debian and RPM

## Development Guidelines

### Security First
This project handles security-critical confidential computing functionality. Always consider:
- Memory safety and secure coding practices
- Cryptographic operations and key management
- TEE attestation and verification flows
- TPM/vTPM interaction and secret storage

### Build System
- Use `cargo` commands for standard Rust development
- Install required system dependencies: Intel SGX, TPM tools (`tpm2-tools`, `libtss2-dev`, `libtdx-attest-dev`)
- Build both debug and release versions: `cargo build --release`
- Package for distribution: `cargo deb` and `cargo generate-rpm`

### Testing Strategy
- Run unit tests: `cargo test`
- Integration testing with cloud providers
- TEE functionality testing on supported hardware
- Certificate and attestation verification testing

### Code Quality
The project maintains high code quality standards:
- **Formatting**: `cargo fmt --all -- --check`
- **Linting**: `cargo clippy -- -D warnings`
- **Dependency audit**: `cargo machete` and `cargo deny check`
- **Security**: Regular dependency updates and vulnerability scanning

### Pre-commit Hooks
Extensive pre-commit hooks are configured covering:
- Rust formatting and linting
- Markdown and YAML validation
- Ansible playbook linting
- Typo checking and text formatting
- License and security compliance

## Key Concepts

### Confidential Computing
- **TEE (Trusted Execution Environment)**: Hardware-based memory encryption (AMD SEV-SNP, Intel TDX)
- **TPM (Trusted Platform Module)**: Secure storage and attestation capabilities
- **IMA (Integrity Measurement Architecture)**: Linux kernel module for executable measurement

### Attestation Flow
1. VM boots with TEE and TPM enabled
2. Agent collects measurements and attestations
3. Client verifies trustworthiness remotely
4. Applications can be deployed securely

### Verification Process
- TEE attestation proves hardware security
- TPM attestation verifies measurement log integrity
- IMA log comparison against snapshots detects unauthorized changes

## Common Tasks

### Adding New Features
1. Identify the appropriate crate (`agent`, `cli`, `client`, etc.)
2. Follow Rust best practices and security guidelines
3. Add comprehensive tests including security scenarios
4. Update relevant documentation and examples

### Debugging Issues
- Check agent logs in confidential VMs
- Verify TEE and TPM functionality
- Validate certificate chains and attestations
- Test across different cloud providers and TEE types

### Cloud Provider Support
- **AWS**: AMD SEV support with Ubuntu/RHEL images
- **Azure**: Intel TDX and AMD SEV with CVM images
- **GCP**: Both TEE types with standard images
- Each provider has specific configuration requirements

## Dependencies and Tools

### System Requirements
- Intel SGX SDK and drivers
- TPM 2.0 tools and libraries
- LUKS for encrypted containers
- Cloud provider CLI tools (aws, az, gcloud)

### Rust Dependencies
- Security-focused crates: `tee_attestation`, `tpm_quote`, `ratls`
- Cryptographic libraries: `aes-gcm`, `ecdsa`, `p256`, `x509-cert`
- Web framework: `actix-web` with TLS support
- Serialization: `serde` and `serde_json`

## Best Practices

1. **Always validate inputs** from external sources and network requests
2. **Use secure defaults** for cryptographic operations and configurations
3. **Test on actual TEE hardware** when possible, not just emulation
4. **Follow the principle of least privilege** for TPM and system access
5. **Maintain backward compatibility** for existing VM deployments
6. **Document security implications** of any changes to attestation flows

## Troubleshooting

### Common Issues
- TEE availability and configuration on cloud instances
- TPM device access and permissions
- Certificate validation and chain of trust
- Network connectivity for attestation services

### Debugging Commands
```bash
# Check TEE capabilities
sudo dmesg | grep -E "SEV|TDX"

# Verify TPM functionality
sudo tpm2_getrandom 32

# Test agent connectivity
curl -k https://instance:5555/version

# Validate IMA measurements
sudo cat /sys/kernel/security/ima/ascii_runtime_measurements
```

## Resources

- [Confidential Computing Consortium](https://confidentialcomputing.io/)
- [AMD SEV-SNP Documentation](https://developer.amd.com/sev/)
- [Intel TDX Documentation](https://www.intel.com/content/www/us/en/developer/tools/trust-domain-extensions/overview.html)
- [TPM 2.0 Specification](https://trustedcomputinggroup.org/resource/tpm-library-specification/)