// Default FRF action policy.
// Cedar governs mutation actions (Publish, Subscribe).
// Visibility is governed by Keto (AuthzProvider) — do not conflate.
//
// Replace with a policy loaded from a policy store in production.

permit(principal, action == Action::"Publish", resource);
permit(principal, action == Action::"Subscribe", resource);
