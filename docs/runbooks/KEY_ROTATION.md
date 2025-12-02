# WRAITH Protocol Key Rotation Runbook

**Version:** 1.0
**Last Updated:** 2025-12-01
**Maintainer:** WRAITH Security Team

---

## Overview

This runbook provides procedures for rotating cryptographic keys in WRAITH Protocol deployments. Regular key rotation is critical for maintaining forward secrecy and limiting the impact of potential key compromises.

**Key Types:**
- **Node Keys (Ed25519)**: Long-term identity keys
- **Session Keys (ChaCha20-Poly1305)**: Ephemeral keys per session
- **Ratchet Keys**: Auto-rotated symmetric keys
- **File Encryption Keys (Argon2id)**: For key storage encryption

---

## Rotation Schedule

**Recommended Schedule:**
- **Node Keys**: Annual or on compromise
- **Session Keys**: Per-session (automatic)
- **Ratchet Keys**: Every 2 min or 1M packets (automatic)
- **File Encryption Passwords**: Every 90 days

---

## Node Key Rotation

### Prerequisites

- [ ] Backup current keys
- [ ] Schedule maintenance window (5-10 minutes downtime)
- [ ] Notify peers of key change
- [ ] Prepare rollback plan

### Procedure

#### 1. Backup Current Keys

```bash
# Create backup directory
sudo mkdir -p /backup/wraith/keys/$(date +%Y%m%d)

# Backup current keys
sudo cp /var/lib/wraith/keys/node_key.enc \
  /backup/wraith/keys/$(date +%Y%m%d)/node_key.enc.backup

# Verify backup
sudo sha256sum /var/lib/wraith/keys/node_key.enc \
  /backup/wraith/keys/$(date +%Y%m%d)/node_key.enc.backup
```

#### 2. Generate New Keys

```bash
# Generate new keypair
sudo -u wraith wraith keygen -o /var/lib/wraith/keys/new_node_key.enc

# Set permissions
sudo chmod 600 /var/lib/wraith/keys/new_node_key.enc
sudo chown wraith:wraith /var/lib/wraith/keys/new_node_key.enc

# Verify new keys
wraith key info /var/lib/wraith/keys/new_node_key.enc
```

#### 3. Update Configuration

```bash
# Stop service
sudo systemctl stop wraith

# Update config to use new keys
sudo sed -i 's/node_key.enc/new_node_key.enc/' /etc/wraith/config.toml

# Verify config
wraith config --validate /etc/wraith/config.toml
```

#### 4. Publish New Public Key

```bash
# Extract public key
NEW_PUBKEY=$(wraith key pubkey /var/lib/wraith/keys/new_node_key.enc)

# Publish to DHT (if supported)
wraith dht publish --key $NEW_PUBKEY

# Update DNS TXT record (optional)
# Add TXT record: wraith-pubkey=<NEW_PUBKEY>
```

#### 5. Start Service with New Keys

```bash
# Start service
sudo systemctl start wraith

# Verify startup
sudo systemctl status wraith

# Check node ID changed
wraith status
```

#### 6. Verify Connectivity

```bash
# Wait for DHT propagation (1-5 minutes)
sleep 60

# Check peer connections
wraith peers

# Test file transfer
wraith send test.dat <peer_id>

# Monitor logs for errors
sudo journalctl -u wraith -f
```

#### 7. Cleanup Old Keys

```bash
# Wait 24 hours for all peers to update

# Securely delete old keys
sudo shred -vfz -n 10 /var/lib/wraith/keys/node_key.enc

# Verify deletion
ls -la /var/lib/wraith/keys/
```

---

## Session Key Rotation

### Automatic Rotation

Session keys are automatically rotated using the Double Ratchet protocol. No manual intervention required.

**Verification:**

```bash
# Check ratchet state
wraith session <session_id> --show-ratchet

# Verify rotation interval
# Expected: Every 2 minutes or 1M packets
```

### Manual Session Rekey

```bash
# Force session rekey
wraith session <session_id> --rekey

# Verify new keys
sudo journalctl -u wraith | grep "session rekey"
```

---

## File Encryption Password Rotation

### Procedure

#### 1. Decrypt Keys with Old Password

```bash
# Decrypt current keys
wraith key decrypt /var/lib/wraith/keys/node_key.enc -o /tmp/node_key.plain

# Verify decryption
wraith key info /tmp/node_key.plain
```

#### 2. Re-encrypt with New Password

```bash
# Re-encrypt with new password
wraith key encrypt /tmp/node_key.plain -o /var/lib/wraith/keys/node_key_new.enc

# Verify new encryption
wraith key info /var/lib/wraith/keys/node_key_new.enc
```

#### 3. Update and Cleanup

```bash
# Stop service
sudo systemctl stop wraith

# Replace old key file
sudo mv /var/lib/wraith/keys/node_key_new.enc /var/lib/wraith/keys/node_key.enc

# Securely delete plaintext
sudo shred -vfz -n 10 /tmp/node_key.plain

# Update systemd secret (if using)
sudo systemd-creds encrypt - /etc/wraith/node_key.cred < /var/lib/wraith/keys/node_key.enc

# Start service
sudo systemctl start wraith
```

---

## Emergency Key Rotation (Compromise)

### Immediate Actions

```bash
# 1. IMMEDIATELY stop service
sudo systemctl stop wraith

# 2. Isolate from network
sudo iptables -A INPUT -j DROP
sudo iptables -A OUTPUT -j DROP

# 3. Generate new keys
sudo -u wraith wraith keygen -o /var/lib/wraith/keys/emergency_key.enc

# 4. Update config
sudo sed -i 's/node_key.enc/emergency_key.enc/' /etc/wraith/config.toml

# 5. Revoke compromised keys (if DHT supports it)
wraith key revoke <compromised_key_id>

# 6. Remove network isolation
sudo iptables -D INPUT -j DROP
sudo iptables -D OUTPUT -j DROP

# 7. Start service
sudo systemctl start wraith

# 8. Notify all peers
echo "Node keys rotated due to compromise. New pubkey: $(wraith key pubkey)" | \
  mail -s "URGENT: Key Rotation" peers@wraith.example
```

---

## Bulk Key Rotation (Fleet)

For rotating keys across multiple nodes:

### Using Ansible

```yaml
# ansible-playbook rotate-keys.yml
---
- name: Rotate WRAITH node keys
  hosts: wraith_nodes
  become: yes
  tasks:
    - name: Backup current keys
      copy:
        src: /var/lib/wraith/keys/node_key.enc
        dest: /backup/wraith/node_key.enc.{{ ansible_date_time.date }}
        remote_src: yes

    - name: Generate new keys
      command: wraith keygen -o /var/lib/wraith/keys/new_node_key.enc
      become_user: wraith

    - name: Stop service
      systemd:
        name: wraith
        state: stopped

    - name: Update config
      replace:
        path: /etc/wraith/config.toml
        regexp: 'node_key.enc'
        replace: 'new_node_key.enc'

    - name: Start service
      systemd:
        name: wraith
        state: started

    - name: Verify status
      systemd:
        name: wraith
        state: started
      register: service_status

    - name: Cleanup old keys (after 24h)
      file:
        path: /var/lib/wraith/keys/node_key.enc
        state: absent
      when: rotate_date is defined and (ansible_date_time.epoch|int - rotate_date|int) > 86400
```

---

## Verification Checklist

### Post-Rotation Checks

- [ ] **Service Status**
  ```bash
  sudo systemctl status wraith
  ```

- [ ] **Node ID Changed**
  ```bash
  wraith status | grep "Node ID"
  ```

- [ ] **Peer Connectivity**
  ```bash
  wraith peers | wc -l  # Should be > 0
  ```

- [ ] **File Transfer Works**
  ```bash
  wraith send test.dat <peer_id>
  ```

- [ ] **Old Keys Deleted**
  ```bash
  ls -la /var/lib/wraith/keys/ | grep -v new
  ```

- [ ] **Logs Clean**
  ```bash
  sudo journalctl -u wraith -n 100 | grep -i "error\|fail"
  ```

---

## Rollback Procedure

If rotation fails:

```bash
# 1. Stop service
sudo systemctl stop wraith

# 2. Restore old keys
sudo cp /backup/wraith/keys/$(date +%Y%m%d)/node_key.enc.backup \
  /var/lib/wraith/keys/node_key.enc

# 3. Restore old config
sudo sed -i 's/new_node_key.enc/node_key.enc/' /etc/wraith/config.toml

# 4. Start service
sudo systemctl start wraith

# 5. Verify rollback
wraith status
sudo systemctl status wraith
```

---

## Troubleshooting

**Keys don't decrypt:**
- Verify password is correct
- Check file permissions (600)
- Verify file not corrupted (sha256sum)

**Peers can't connect after rotation:**
- Wait 5-10 minutes for DHT propagation
- Check public key published correctly
- Verify firewall rules still allow connections

**Service won't start with new keys:**
- Check key file exists and is readable
- Verify config syntax: `wraith config --validate`
- Check logs: `sudo journalctl -u wraith -n 100`

---

## Best Practices

1. **Always backup** before rotation
2. **Test in staging** before production
3. **Rotate during low-traffic** periods
4. **Monitor closely** after rotation
5. **Keep audit trail** of all rotations
6. **Automate** where possible
7. **Use HSM** for high-security environments

---

## Audit Log

Maintain a rotation log:

```bash
# /var/log/wraith/key-rotation.log
2025-12-01 10:00:00 UTC | Node Key Rotation | node-01 | Success | admin@example.com
2025-12-01 10:15:00 UTC | Node Key Rotation | node-02 | Success | admin@example.com
2025-12-01 10:30:00 UTC | Node Key Rotation | node-03 | Failed | admin@example.com | Rollback completed
```

---

## Security Considerations

1. **Never transmit** private keys over network
2. **Always encrypt** keys at rest
3. **Use strong passwords** (20+ characters, random)
4. **Limit access** to key files (chmod 600)
5. **Audit key operations** (who, what, when)
6. **Secure deletion** (shred -n 10)
7. **Offsite backup** of encrypted keys

---

## Compliance

For regulatory compliance (SOC 2, ISO 27001, etc.):

- [ ] Document rotation policy
- [ ] Maintain rotation schedule
- [ ] Log all rotations
- [ ] Review logs quarterly
- [ ] Test rollback procedures
- [ ] Train personnel on procedures
- [ ] Include in disaster recovery plan
