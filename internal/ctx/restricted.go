package ctx

import (
	"fmt"
	"sort"

	gostarlark "go.starlark.net/starlark"
)

// RestrictedCtxValue wraps a CtxValue with a per-method allow-list. Attr
// returns (nil, nil) for any name not in the allow-list, which Starlark
// interprets as attribute-not-found. Used for shell.star execution where
// mutating ctx operations must not be available.
type RestrictedCtxValue struct {
	inner     *CtxValue
	allowList map[string]bool
}

// Compile-time interface checks.
var (
	_ gostarlark.Value    = (*RestrictedCtxValue)(nil)
	_ gostarlark.HasAttrs = (*RestrictedCtxValue)(nil)
)

// NewRestricted constructs a RestrictedCtxValue backed by a fresh CtxValue.
// Only names present in allowed are accessible via Attr.
func NewRestricted(caps *Capabilities, allowed []string) *RestrictedCtxValue {
	set := make(map[string]bool, len(allowed))
	for _, n := range allowed {
		set[n] = true
	}
	return &RestrictedCtxValue{inner: New(caps), allowList: set}
}

// String implements starlark.Value.
func (r *RestrictedCtxValue) String() string { return "<ctx>" }

// Type implements starlark.Value.
func (r *RestrictedCtxValue) Type() string { return "ctx" }

// Freeze implements starlark.Value.
func (r *RestrictedCtxValue) Freeze() {}

// Truth implements starlark.Value.
func (r *RestrictedCtxValue) Truth() gostarlark.Bool { return gostarlark.True }

// Hash implements starlark.Value. ctx objects are not hashable.
func (r *RestrictedCtxValue) Hash() (uint32, error) {
	return 0, fmt.Errorf("unhashable type: ctx")
}

// Attr implements starlark.HasAttrs. Returns (nil, nil) for names not in the
// allow-list so Starlark reports attribute-not-found rather than a Go error.
func (r *RestrictedCtxValue) Attr(name string) (gostarlark.Value, error) {
	if !r.allowList[name] {
		return nil, nil
	}
	return r.inner.Attr(name)
}

// AttrNames implements starlark.HasAttrs. Returns only the allowed names,
// sorted, so dir(ctx) in Starlark reflects the restricted surface.
func (r *RestrictedCtxValue) AttrNames() []string {
	names := make([]string, 0, len(r.allowList))
	for k := range r.allowList {
		names = append(names, k)
	}
	sort.Strings(names)
	return names
}
