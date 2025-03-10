# BitVault Threat Model

This document presents a systematic analysis of security threats to BitVault, evaluating how our security architecture mitigates these threats and identifying residual risks. The threat model focuses on realistic attack scenarios specifically targeting a Bitcoin wallet with a 2-of-3 multisignature architecture.

## 1. Threat Modeling Approach

### Methodology

This threat model follows a structured approach:

1. **Identify assets**: Define what we're protecting
2. **Identify threat actors**: Who might attack the system
3. **Enumerate attack vectors**: How attacks might be executed
4. **Analyze security boundaries**: Evaluate how our architecture resists attacks
5. **Identify mitigations**: How we address specific threats
6. **Assess residual risks**: What risks remain after mitigations

### Critical Assets to Protect

In order of priority:

1. **Private keys**: The cryptographic keys that control Bitcoin funds
2. **Seed phrases/recovery information**: Material that can regenerate private keys
3. **Transaction integrity**: Ensuring transactions execute as intended by the user
4. **User authentication credentials**: Passwords, biometrics, and session data
5. **Wallet metadata**: Address information, transaction history, contacts
6. **User privacy**: Transaction patterns, balance information, identity correlation

## 2. Threat Actors

### Sophisticated Targeted Attackers

**Capabilities**:
- Advanced technical skills and resources
- Ability to develop custom malware
- Knowledge of Bitcoin security models
- Potentially nation-state level capabilities
- Willing to invest significant resources for high-value targets

**Motivations**:
- Financial gain from stealing high-value Bitcoin holdings
- Targeting specific individuals or organizations
- Intelligence gathering

**Examples**:
- Nation-state actors targeting specific users
- Advanced persistent threats (APTs)
- Sophisticated criminal organizations

### Opportunistic Attackers

**Capabilities**:
- Moderate technical skills
- Use of existing attack tools and frameworks
- Limited resources for sustained attacks
- Willing to invest moderate effort for potential gain

**Motivations**:
- Financial gain from stealing any accessible Bitcoin
- Opportunistic targeting based on perceived vulnerability
- Volume-based approach

**Examples**:
- Malware developers targeting cryptocurrency users
- Phishing campaign operators
- Criminal groups with some technical capability

### Malicious Insiders

**Capabilities**:
- Legitimate access to some system components
- Knowledge of system architecture and potential weaknesses
- Ability to operate from positions of trust

**Motivations**:
- Financial gain
- Sabotage
- Espionage

**Examples**:
- Compromised developers with code access
- Supply chain attackers
- Malicious service providers

### Physical Attackers

**Capabilities**:
- Physical access to user devices
- Ability to steal or tamper with hardware
- Potential for coercion or social engineering

**Motivations**:
- Theft of Bitcoin assets
- Access to sensitive information
- Bypass of digital security measures

**Examples**:
- Device thieves
- Border/customs officials
- Coercive actors ("$5 wrench attack")

## 3. Attack Vectors

### Endpoint Compromise

**Malware-Based Attacks**:
- Keyloggers capturing passwords/seed phrases
- Screen capture of sensitive information
- Memory scraping for private keys
- Transaction tampering (e.g., address replacement)
- Backdoor installation for persistent access

**Operating System Level Attacks**:
- Exploitation of OS vulnerabilities
- Privilege escalation to access protected resources
- Compromise of system cryptographic services
- Subversion of process isolation mechanisms

**Application Level Attacks**:
- Exploitation of application vulnerabilities
- Dependencies/supply chain compromises
- Development environment compromises
- Build system attacks

### Cryptographic Attacks

**Implementation Attacks**:
- Side-channel attacks against cryptographic operations
- Timing attacks on cryptographic implementations
- Fault injection attacks
- Weak random number generation exploitation

**Protocol Attacks**:
- Transaction malleability exploitation
- Signature algorithm weaknesses
- Nonce reuse vulnerabilities
- Replay attacks

### Social Engineering

**Direct User Manipulation**:
- Phishing for seed phrases or passwords
- Fake wallet applications
- Social engineering to authorize malicious transactions
- Support scams

**Indirect Attacks**:
- Supply chain compromise
- Fraudulent updates
- Malicious dependencies

### Physical Security Threats

**Device Theft/Loss**:
- Theft of device with wallet software
- Loss of backup materials
- Extraction of keys from stolen devices

**Coercion Attacks**:
- Forced disclosure of authentication credentials
- Compelled transaction signing under duress
- Legal compulsion to disclose keys

### Network-Based Attacks

**Man-in-the-Middle Attacks**:
- Intercepting communications between wallet components
- Tampering with transaction data in transit
- DNS hijacking to redirect to malicious services

**Denial of Service**:
- Preventing access to wallet functionality
- Blocking critical operations during important market events
- Resource exhaustion attacks

## 4. Security Boundary Analysis

### Process Isolation Boundary

**Threats Addressed**:
- Memory access to private keys from compromised UI process
- Keyloggers capturing key material
- Screen capture of private data
- Malicious code in UI process accessing keys

**Implementation**:
- Separate OS processes for UI and secure operations
- Restricted permissions for secure process
- Explicit IPC for all cross-boundary communication
- Message authentication for all IPC

**Effectiveness against Threat Actors**:
- **Sophisticated Targeted Attackers**: Partial - May develop kernel-level attacks to bypass
- **Opportunistic Attackers**: Strong - Prevents common malware approaches
- **Malicious Insiders**: Partial - Design provides separation but implementation vulnerabilities possible
- **Physical Attackers**: Limited - Physical access may enable bypass

### Authentication Boundary

**Threats Addressed**:
- Unauthorized access to wallet functionality
- Unwanted transaction authorization
- Session hijacking

**Implementation**:
- Multi-factor authentication for sensitive operations
- Time-limited sessions with explicit reauthentication
- Platform-specific secure authentication integration
- Escalating authentication for higher-risk operations

**Effectiveness against Threat Actors**:
- **Sophisticated Targeted Attackers**: Moderate - Advanced keyloggers still a risk
- **Opportunistic Attackers**: Strong - Prevents automated attacks
- **Malicious Insiders**: Strong - Cannot easily bypass authentication
- **Physical Attackers**: Moderate - Device theft after authentication still a risk

### Multisignature Security Boundary

**Threats Addressed**:
- Compromise of a single key/device
- Unauthorized transaction signing
- Loss of a single key/device

**Implementation**:
- 2-of-3 signature requirement for all transactions
- Distribution of keys across different security domains
- Different authorization mechanisms for each key

**Effectiveness against Threat Actors**:
- **Sophisticated Targeted Attackers**: Strong - Must compromise multiple systems
- **Opportunistic Attackers**: Very Strong - Unlikely to compromise multiple keys
- **Malicious Insiders**: Strong - Cannot unilaterally authorize transactions
- **Physical Attackers**: Strong - Physical access to one device insufficient

### Platform Security Integration

**Threats Addressed**:
- Platform-specific vulnerabilities
- Secure storage weaknesses
- Authentication bypass

**Implementation**:
- Secure Enclave integration on iOS
- Keystore integration on Android
- Platform-specific secure storage APIs
- Biometric integration where available

**Effectiveness against Threat Actors**:
- **Sophisticated Targeted Attackers**: Varies by platform - Strongest on iOS
- **Opportunistic Attackers**: Strong - Platform security effective against common attacks
- **Malicious Insiders**: Moderate - Platform boundaries provide some protection
- **Physical Attackers**: Moderate - Platform security provides some physical protection

## 5. Threat Scenarios and Mitigations

### Scenario 1: Malware-Infected Primary Device

**Attack Path**:
1. User's primary device infected with sophisticated malware
2. Malware attempts to access private keys or manipulate transactions
3. Attacker tries to authorize fraudulent transactions

**Mitigations**:
- Process isolation prevents direct key access from UI process
- 2-of-3 multisig requires second device approval
- Transaction details shown on secondary device for verification
- Security policy enforcement in secure process
- Address validation and comparison on both devices

**Residual Risks**:
- Kernel-level malware could potentially bypass process isolation
- Sophisticated screen/input capture might trick user into approving malicious transaction
- Malware persistence might enable long-term attacks

### Scenario 2: Theft of Primary Device

**Attack Path**:
1. Attacker physically steals user's primary device
2. Attempts to authenticate and access wallet
3. Tries to extract keys or authorize transactions

**Mitigations**:
- Authentication required to access wallet
- Secure storage of keys with platform protection
- 2-of-3 multisig requires secondary device
- Biometric authentication where available
- Session timeouts limit window of opportunity
- Remote wipe capabilities (where implemented)

**Residual Risks**:
- Device stolen while in authenticated state
- Sophisticated hardware attacks against some platforms
- Forensic recovery of keys if implementation flaws exist

### Scenario 3: Phishing Attack

**Attack Path**:
1. User receives convincing phishing communication
2. Attacker attempts to obtain seed phrases or authentication credentials
3. Alternatively, tricks user into installing fake wallet app

**Mitigations**:
- Guided backup procedures with security education
- Clear wallet verification procedures
- Open source code allows verification
- Multiple key recovery requirements
- No seed phrase entry in primary UI

**Residual Risks**:
- User error in recording/storing seed phrases
- Highly convincing phishing still effective against some users
- Recovery process susceptibility to social engineering

### Scenario 4: Supply Chain Attack

**Attack Path**:
1. Attacker compromises development dependencies or build system
2. Malicious code is incorporated into wallet
3. Trojan functionality targets keys or transactions

**Mitigations**:
- Minimized dependencies, especially in core module
- Reproducible builds (where implemented)
- Open source code enables scrutiny
- Code signing and update verification
- Security boundary architecture limits damage potential

**Residual Risks**:
- Sophisticated supply chain attacks may still succeed
- Review process may miss subtle backdoors
- Dependency vulnerabilities may not be detected promptly

### Scenario 5: Transaction Manipulation Attack

**Attack Path**:
1. Attacker compromises UI process
2. Attempts to modify transaction details before signing
3. Tries to change recipient address or amount

**Mitigations**:
- Transaction details verified on secondary device
- Address verification and comparison
- Policy limits on transaction amounts
- PSBT signing model with explicit verification
- Hash verification of transaction details

**Residual Risks**:
- User inattention during verification
- Similar-looking addresses may not be noticed
- Verification fatigue for routine transactions

### Scenario 6: Coercion Attack

**Attack Path**:
1. Attacker physically threatens user
2. Forces user to unlock wallet and authorize transactions
3. Compels transfer of funds

**Mitigations**:
- Duress wallet option (post-MVP)
- Timelocked recovery transactions (post-MVP)
- Threshold signature requiring physically separated key shares
- Policy-based spending limits

**Residual Risks**:
- Sophisticated coercion may overcome protections
- Immediate transfers still possible with sufficient coercion
- Limited protection in minimum viable product

## 6. Platform-Specific Threat Scenarios

### Linux-Specific Threats

**Scenario L1: Memory Dumping Attack via Process Injection**
- **Attack Path:** 
  1. Attacker gains elevated privileges on the system
  2. Uses ptrace to attach to secure process
  3. Dumps memory to extract key material
  4. Extracts private key shares from memory dump

- **Platform Factors:** 
  - Linux process debugging capabilities
  - Memory inspection tools
  - Default ptrace permissions

- **Specialized Mitigations:**
  - Set PR_SET_DUMPABLE to 0 using prctl()
  - Implement ptrace restrictions with seccomp
  - Use process namespaces to isolate memory spaces
  - Apply YAMA ptrace_scope restriction enforcement
  - Memory encryption for key material with session keys

- **Residual Risk Assessment:**
  - Kernel-level attackers can still access memory
  - Advanced cold boot attacks remain possible
  - Hardware-level memory access not fully mitigated

**Scenario L2: D-Bus Service Impersonation**
- **Attack Path:** 
  1. Malicious process registers on D-Bus with similar name
  2. Intercepts communication with secure storage services
  3. Captures credentials or sensitive data
  4. Potentially modifies secure storage content

- **Platform Factors:** 
  - Linux IPC mechanisms
  - D-Bus service discovery
  - Authentication mechanisms

- **Specialized Mitigations:**
  - D-Bus service validation through signature verification
  - Secure process pinning to specific D-Bus unique names
  - Secondary validation of service authentication
  - Fall back to encrypted file storage if D-Bus integrity uncertain
  - Service attestation through kernel-mediated verification

- **Residual Risk Assessment:**
  - Complex D-Bus security validation may be bypassed
  - Privileged attackers can still intercept system services
  - Fallback mechanisms may have different security properties

**Scenario L3: X11 Screen Capture and Input Hijacking**
- **Attack Path:** 
  1. Malware exploits X11 security model to capture screen contents
  2. Captures sensitive data displayed on screen during wallet usage
  3. Potentially injects keystrokes to manipulate wallet operations
  4. Modifies displayed transaction information

- **Platform Factors:** 
  - X11 security model limitations
  - Input handling accessibility
  - Screen sharing capabilities

- **Specialized Mitigations:**
  - Wayland preference where available for improved isolation
  - Secure input paths for password entry (kernel direct where possible)
  - Secure attention sequence implementation (custom key combination)
  - Visual transaction signing with verification codes
  - Secondary device confirmation for high-value transactions

- **Residual Risk Assessment:**
  - Display server compromises difficult to fully mitigate
  - Wayland not universally available or mature
  - Advanced screen capture may bypass protections

### Android-Specific Threats

**Scenario A1: Compromised TEE/Keystore Implementation**
- **Attack Path:** 
  1. Device uses TEE with known vulnerabilities
  2. Attacker exploits TEE vulnerability to extract key material
  3. Compromises Android Keystore implementation
  4. Bypasses hardware-backed security guarantees

- **Platform Factors:** 
  - Vendor-specific TEE implementations
  - Android fragmentation
  - Uneven security updates

- **Specialized Mitigations:**
  - Hardware attestation verification at runtime
  - Security level downgrade with clear user notification
  - Enhanced software protections when hardware security questionable
  - Key share distribution adjusted based on security capability
  - Behavioral analysis to detect potential TEE compromise

- **Residual Risk Assessment:**
  - Cannot fully validate proprietary TEE implementations
  - Hardware attestation can potentially be spoofed
  - Security downgrade reduces protection but enables awareness

**Scenario A2: Screen Overlay Attack During Transaction**
- **Attack Path:** 
  1. Malicious app obtains overlay permission
  2. Activates overlay during transaction authorization
  3. Modifies displayed transaction information
  4. Tricks user into authorizing different transaction

- **Platform Factors:** 
  - Android overlay capabilities
  - Permission model changes across versions
  - User interface design

- **Specialized Mitigations:**
  - FLAG_SECURE to prevent screenshots and screen recording
  - Verification that no overlay is active during sensitive operations
  - Out-of-band transaction verification (secondary device or channel)
  - Visual security indicators that cannot be easily spoofed
  - Contextual authentication linked to transaction details

- **Residual Risk Assessment:**
  - Some overlay attacks difficult to detect on all Android versions
  - User attention to verification remains critical
  - Advanced malware might bypass overlay detection

**Scenario A3: Accessibility Service Exploitation**
- **Attack Path:** 
  1. Malicious app gains accessibility service permissions
  2. Accessibility service reads screen content including sensitive data
  3. Service injects fake taps or gestures
  4. Manipulates wallet operations through legitimately granted permissions

- **Platform Factors:** 
  - Android accessibility framework
  - Broad permissions granted to accessibility services
  - Legitimate use cases for accessibility

- **Specialized Mitigations:**
  - Detection of active accessibility services during sensitive operations
  - Custom input methods for critical data entry
  - Visual cryptographic techniques for secure display
  - Transaction confirmation through secondary channels
  - Security warnings when accessibility services are enabled

- **Residual Risk Assessment:**
  - Cannot block accessibility services (usability requirement)
  - Warning fatigue may lead to ignored security alerts
  - Sophisticated attacks might bypass mitigations

### iOS-Specific Threats

**Scenario I1: Secure Enclave Side-Channel Attack**
- **Attack Path:** 
  1. Sophisticated attacker exploits side-channel vulnerabilities
  2. Targets Secure Enclave implementation flaws
  3. Extracts key material through timing or power analysis
  4. Bypasses hardware security guarantees

- **Platform Factors:** 
  - Hardware security module implementation details
  - Proprietary security architecture
  - Limited visibility into implementation

- **Specialized Mitigations:**
  - Distribute key shares across contexts (not all in Secure Enclave)
  - Implement additional software obfuscation for sensitive operations
  - Regular rotation of keys used in threshold scheme
  - Transaction monitoring for anomalous patterns
  - Security updates monitoring for Secure Enclave vulnerabilities

- **Residual Risk Assessment:**
  - Side-channel vulnerabilities may be unpublished
  - Limited ability to enhance proprietary hardware security
  - Implementation details controlled by Apple

**Scenario I2: Manipulated Device State (Jailbreak)**
- **Attack Path:** 
  1. Device jailbroken to bypass security restrictions
  2. Security boundaries compromised by root access
  3. Attacker gains ability to inspect application memory
  4. Bypasses platform security controls

- **Platform Factors:** 
  - iOS security model heavily dependent on system integrity
  - Jailbreaking techniques evolve with OS versions
  - System integrity compromised

- **Specialized Mitigations:**
  - Advanced jailbreak detection using multiple methods
  - Security downgrade with explicit user notification
  - Device attestation through server validation
  - Remote policy enforcement with attestation requirements
  - Backup verification on separate trusted device

- **Residual Risk Assessment:**
  - Jailbreak detection is an ongoing arms race
  - Advanced attackers can bypass jailbreak detection
  - Prevention limited, focus on detection and notification

**Scenario I3: Swift/Objective-C Runtime Manipulation**
- **Attack Path:** 
  1. Attacker uses method swizzling or runtime modification
  2. Intercepts critical method calls in the application
  3. Modifies behavior of security-critical functions
  4. Bypasses application-level security checks

- **Platform Factors:** 
  - Dynamic language runtime
  - Method swizzling capabilities
  - Objective-C runtime flexibility

- **Specialized Mitigations:**
  - Code signing and integrity verification
  - Anti-swizzling techniques for critical methods
  - Redundant security checks through different paths
  - Use of Swift over Objective-C where possible
  - Critical path validation through cryptographic means

- **Residual Risk Assessment:**
  - Runtime manipulation difficult to completely prevent
  - Advanced attackers can modify app behavior at runtime
  - Defense-in-depth approach provides multiple validation layers

## 7. Residual Risk Assessment

### High-Risk Residual Threats

**Advanced Persistent Threats**:
- Sophisticated attackers with significant resources
- Zero-day vulnerabilities in platform security
- Long-term monitoring and advanced techniques
- **Mitigation Strategy**: Multisignature architecture forces compromise of multiple systems

**Implementation Vulnerabilities**:
- Bugs in security-critical code
- Side-channel leaks in cryptographic operations
- Memory safety issues despite Rust's protections
- **Mitigation Strategy**: Rigorous security review, testing, and external audit

**User Error and Social Engineering**:
- Improper backup procedures
- Falling for phishing attacks
- Verification fatigue leading to missed attack indicators
- **Mitigation Strategy**: User education, guided workflows, and verification procedures

### Medium-Risk Residual Threats

**Physical Device Compromise**:
- Advanced forensic analysis of stolen devices
- Cold boot attacks against RAM
- Hardware modification attacks
- **Mitigation Strategy**: Encryption, memory protection, and key isolation

**Supply Chain Risks**:
- Compromised dependencies
- Build system attacks
- Malicious development tools
- **Mitigation Strategy**: Dependency minimization, reproducible builds, code review

**Transaction Manipulation**:
- Address replacement attacks
- Fee manipulation
- Race condition exploitation
- **Mitigation Strategy**: Verification on secondary device, clear transaction display

### Accepted Limitations

**Usability vs. Security Trade-offs**:
- Perfect security would make the system unusable
- Some convenience features inherently reduce security
- Users may disable security features for convenience

**Platform Security Reliance**:
- Dependency on platform security guarantees
- Limited ability to verify platform security claims
- Varying security levels across platforms

**Resource Constraints**:
- Not all security measures can be implemented for MVP
- Some advanced protections deferred to future versions
- Security monitoring and response capabilities limited

## 8. Security Testing Priorities

### Critical Testing Areas

1. **Security Boundary Effectiveness**:
   - Process isolation validation
   - IPC security testing
   - Memory protection verification
   - Authentication bypass attempts

2. **Cryptographic Implementation**:
   - Key generation and management
   - Transaction signing correctness
   - Side-channel analysis
   - Random number generation quality

3. **Multisignature Operations**:
   - Transaction flow integrity
   - PSBT handling
   - Backup and recovery procedures
   - Key separation validation

### Recommended Security Validation Methods

1. **Penetration Testing**:
   - External security review
   - Targeted boundary testing
   - Privilege escalation attempts
   - Attack scenario simulation

2. **Code Review**:
   - Security-focused code review
   - Dependency analysis
   - Cryptographic implementation review
   - Cross-boundary communication review

3. **Fuzzing and Dynamic Analysis**:
   - IPC message fuzzing
   - Invalid input handling
   - API boundary testing
   - Transaction data manipulation

## 9. Threat Model Maintenance

This threat model should be maintained as a living document with:

1. Regular reviews as architecture evolves
2. Updates when new threats are identified
3. Validation against actual security incidents
4. Expansion as new platforms are supported
5. Refinement based on penetration testing results

Security incidents, near-misses, and external developments in Bitcoin security should trigger review of relevant sections. 