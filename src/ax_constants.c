#include <ApplicationServices/ApplicationServices.h>

// Export AX constants as functions to avoid linking issues
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
