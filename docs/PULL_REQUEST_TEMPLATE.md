# Add admin transfer function to forge-vesting

## Summary
Implements the `transfer_admin` function in the forge-vesting contract to allow transferring admin rights to a new address.

## Changes Made
- Added `SameAdmin` error variant in `VestingError` enum
- Implemented `transfer_admin(new_admin: Address)` function in `forge-vesting` contract:
  - Requires authorization from current admin
  - Prevents transferring to the same admin
  - Emits `admin_transferred` event on successful transfer
- Added tests:
  - `test_transfer_admin_success` - verifies admin can be transferred to a new address
  - `test_transfer_admin_by_non_admin_fails` - verifies unauthorized calls fail
  - `test_transfer_admin_to_same_admin_fails` - verifies transferring to same admin fails

## Related Issues
- Fixes #39

## Checklist
- [x] Implementation complete
- [x] Tests added
- [ ] Documentation updated
- [ ] No sensitive information included

## Testing
Tests can be run with: `cargo test --package forge-vesting`