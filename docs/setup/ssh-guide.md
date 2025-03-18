# Setting Up SSH Access to the BitVault Repository

This guide will walk you through setting up SSH authentication to access the BitVault repository on GitHub, with special consideration for the security-focused nature of the project.

## Generate an SSH Key

1. Open your terminal and run the following command to generate a new SSH key using Ed25519 (a more secure algorithm):

   ```bash
   ssh-keygen -t ed25519 -C "your.email@example.com"
   ```

2. When prompted for a file location, press Enter to accept the default location (`~/.ssh/id_ed25519`).

3. You'll be asked to enter a passphrase. Since BitVault is security-focused, it's critical to create a strong passphrase. This adds an essential layer of security if someone gains access to your computer.

## Add Your SSH Key to the SSH Agent

1. Start the SSH agent in the background:

   ```bash
   eval "$(ssh-agent -s)"
   ```

2. Add your private key to the SSH agent:

   ```bash
   ssh-add ~/.ssh/id_ed25519
   ```

3. For added security, consider setting a timeout for your SSH keys:

   ```bash
   # Add key with a 4-hour timeout
   ssh-add -t 4h ~/.ssh/id_ed25519
   ```

## Add Your SSH Key to GitHub

1. Copy your SSH public key to the clipboard:

   ```bash
   cat ~/.ssh/id_ed25519.pub
   ```

   This will output your public key. Copy the entire output string.

2. Go to GitHub and log in to your account.

3. Click on your profile photo in the top-right corner, then click **Settings**.

4. In the user settings sidebar, click **SSH and GPG keys**.

5. Click **New SSH key** or **Add SSH key**.

6. In the "Title" field, add a descriptive label for the key (e.g., "BitVault Development Machine").

7. Paste your key into the "Key" field.

8. Click **Add SSH key**.

9. If prompted, confirm your GitHub password.

## Test Your SSH Connection

Verify that your SSH connection to GitHub is working:

```bash
ssh -T git@github.com
```

You might see a warning about the authenticity of the host. Type "yes" to continue.

If everything is set up correctly, you'll see a message like:
```
Hi username! You've successfully authenticated, but GitHub does not provide shell access.
```

## Clone the BitVault Repository

Now you can clone the repository using SSH:

```bash
git clone git@github.com:BitVaulty/BitVaultWallet.git
```

## Security Considerations for BitVault Development

Due to the security-critical nature of BitVault, follow these additional security practices:

1. **Private Key Security**:
   - Never share your private SSH key with anyone
   - Keep your key passphrase strong and unique
   - Consider hardware security keys (like YubiKey) for SSH authentication

2. **Repository Security**:
   - Be extremely careful when pushing code that deals with cryptographic operations
   - Never commit sensitive test data, private keys, or seeds
   - Ensure all security-critical code undergoes proper review

3. **Development Machine Security**:
   - Keep your development machine updated with security patches
   - Consider using disk encryption (like LUKS on Linux)
   - Be cautious when installing software from untrusted sources

4. **Working with Security-Critical Code**:
   - Pay special attention when modifying code in `bitvault-core`
   - All modifications crossing security boundaries require thorough review
   - Follow the security guidelines in the project documentation

## Working with the Repository

After cloning, set up your development environment as described in the project documentation:

1. Change to the project directory:
   ```bash
   cd BitVaultWallet
   ```

2. Create a new branch for your work:
   ```bash
   git checkout -b your-feature-name
   ```

3. Make your changes, commit them, and push to the repository:
   ```bash
   git add .
   git commit -m "Description of your changes"
   git push origin your-feature-name
   ```

4. Create a pull request through the GitHub web interface.

5. For security-critical changes, explicitly request review from security team members.

## Setting Up Git Hooks for Security

BitVault uses Git hooks to prevent accidental commits of sensitive information:

```bash
# Enable the pre-commit hook
cp .github/hooks/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit

# Test the pre-commit hook
echo "TEST_SEED_PHRASE='word1 word2 word3'" > test_file.txt
git add test_file.txt
git commit -m "Test commit"  # This should be blocked by the hook
```

## Troubleshooting SSH Issues

If you encounter issues with SSH authentication:

1. Ensure your SSH key is added to the SSH agent:
   ```bash
   ssh-add -l
   ```

2. Verify GitHub can see your SSH key:
   ```bash
   ssh -vT git@github.com
   ```

3. Check if you have the correct permissions for the repository. You may need to contact the repository owner to grant you access if you're seeing "Permission denied" errors.

4. If your key is being rejected, ensure you copied the entire public key including the `ssh-ed25519` prefix and the email suffix.
