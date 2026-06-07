# CrossCrypt Security Documentation

## Threat Model

### Assumptions

- User's computer is not compromised at time of encryption
- User chooses a strong password
- Physical access to device is controlled

### Threats Addressed

1. **Unauthorized Access**: Data encrypted at rest
2. **Brute Force**: Argon2id + attempt limiting
3. **Cold Boot**: Memory-only key handling
4. **Evil Maid**: Detectable tampering via checksums

### Threats NOT Addressed

1. **Live System Compromise**: Key in memory while mounted
2. **Side Channel**: Timing/power analysis
3. **Social Engineering**: Password disclosure
4. **Backup Exposure**: Unencrypted backups

## Cryptographic Details

### AES-256-XTS

```
XTS Mode Operation:
C_i = E_K1(P_i ⊕ T_i) ⊕ T_i

Where:
- K1 = First 256 bits of master key
- K2 = Second 256 bits of master key
- T_i = E_K2(i) ⊗ α^j
- i = Sector number
- j = Block within sector
- α = Primitive element of GF(2^128)
```

### Argon2id Parameters

```
Default:
- Time cost (t): 3 iterations
- Memory cost (m): 64 MB
- Parallelism (p): 4 threads

Conservative:
- Time cost (t): 4 iterations
- Memory cost (m): 256 MB
- Parallelism (p): 4 threads
```

### Key Hierarchy

```
User Password
     ↓
Argon2id (salt, params)
     ↓
Key Encryption Key (KEK)
     ↓
Decrypt Master Key
     ↓
AES-256-XTS Key (512 bits)
```

## Security Considerations

### Password Strength

Minimum requirements:
- 8 characters minimum (recommended: 16+)
- Mix of uppercase, lowercase, numbers, symbols
- No dictionary words
- Unique (not reused)

### Memory Security

- Master key stored in locked memory (mlock)
- Cleared on unmount
- Cleared on process termination
- Cleared on lock

### Side Channel Mitigations

- Constant-time comparison for password verification
- Randomized memory layout
- No branching on secret data

## Audit Log

All security-relevant events are logged:
- Mount/unmount operations
- Failed password attempts
- Lock/wipe triggers
- Configuration changes

## Incident Response

### Lost Password

**No recovery possible.** This is by design.
Recommendations:
1. Create backup before encryption
2. Store password in secure password manager
3. Consider Shamir's Secret Sharing for critical data

### Suspected Compromise

1. Immediately lock volume
2. Unmount and physically disconnect
3. Change password (requires re-encryption)
4. Review audit logs

### Wipe Triggered

1. Data is irrecoverably destroyed
2. Volume must be reformatted
3. Restore from backup

## Compliance

### Standards

- FIPS 197 (AES)
- IEEE P1619 (XTS)
- RFC 9106 (Argon2)

### Certifications

Future goals:
- FIPS 140-2 Level 1
- Common Criteria EAL2+

## Vulnerability Disclosure

Please report security issues to:
security@crosscrypt.io

PGP Key: [link]

## Security Changelog

### v0.1.0 (Initial)
- AES-256-XTS encryption
- Argon2id key derivation
- Brute force protection
- Secure wipe
