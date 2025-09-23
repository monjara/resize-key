#include <ApplicationServices/ApplicationServices.h>

// CFStringRef 定数を Rust から参照するための薄いシム
CFStringRef get_kAXFocusedApplicationAttribute(void) {
  return kAXFocusedApplicationAttribute;
}
CFStringRef get_kAXFocusedWindowAttribute(void) {
  return kAXFocusedWindowAttribute;
}
CFStringRef get_kAXPositionAttribute(void) { return kAXPositionAttribute; }
CFStringRef get_kAXSizeAttribute(void) { return kAXSizeAttribute; }
CFStringRef get_kAXTrustedCheckOptionPrompt(void) {
  return kAXTrustedCheckOptionPrompt;
}
