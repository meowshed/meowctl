package ctx

import (
	"fmt"

	gostarlark "go.starlark.net/starlark"
)

// AfterSentinelCtx is passed as the ctx argument when evaluating an after()
// callable. Every attribute access raises a descriptive error directing the
// author to use the platform() builtin instead.
type AfterSentinelCtx struct{}

// Compile-time interface checks.
var (
	_ gostarlark.Value    = (*AfterSentinelCtx)(nil)
	_ gostarlark.HasAttrs = (*AfterSentinelCtx)(nil)
)

// String implements starlark.Value.
func (a *AfterSentinelCtx) String() string { return "<ctx>" }

// Type implements starlark.Value.
func (a *AfterSentinelCtx) Type() string { return "ctx" }

// Freeze implements starlark.Value.
func (a *AfterSentinelCtx) Freeze() {}

// Truth implements starlark.Value.
func (a *AfterSentinelCtx) Truth() gostarlark.Bool { return gostarlark.True }

// Hash implements starlark.Value. ctx objects are not hashable.
func (a *AfterSentinelCtx) Hash() (uint32, error) {
	return 0, fmt.Errorf("unhashable type: ctx")
}

// Attr implements starlark.HasAttrs. Every attribute access returns a
// descriptive error: ctx is not available in after(); use the platform() builtin instead.
func (a *AfterSentinelCtx) Attr(_ string) (gostarlark.Value, error) {
	return nil, fmt.Errorf("ctx is not available in after(); use the platform() builtin instead")
}

// AttrNames implements starlark.HasAttrs.
func (a *AfterSentinelCtx) AttrNames() []string { return nil }
