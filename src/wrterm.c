#include <config.h>
#include "term.h"
#include "lisp.h"

void
syms_of_wrterm (void)
{

  DEFVAR_LISP ("wr-keysym-table", Vwr_keysym_table,
	       doc: /* Hash table of character codes indexed by wr keysym codes.  */);
  Vwr_keysym_table =
    make_hash_table (hashtest_eql, 900, DEFAULT_REHASH_SIZE,
		     DEFAULT_REHASH_THRESHOLD, Qnil, false);

  DEFVAR_LISP ("x-toolkit-scroll-bars", Vx_toolkit_scroll_bars,
	       doc: /* Which toolkit scroll bars Emacs uses, if any.
		       A value of nil means Emacs doesn't use toolkit scroll bars.*/ );
  /* Vx_toolkit_scroll_bars = Qt; */
  Vx_toolkit_scroll_bars = Qnil;

  DEFVAR_BOOL ("x-use-underline-position-properties",
	       x_use_underline_position_properties,
	       doc: /* Non-nil means make use of UNDERLINE_POSITION font properties.
		       A value of nil means ignore them.  If you encounter fonts with bogus
		       UNDERLINE_POSITION font properties, set this to nil.  You can also use
		       `underline-minimum-offset' to override the font's UNDERLINE_POSITION for
		       small font display sizes.  */);
  x_use_underline_position_properties = true;

  DEFVAR_BOOL ("x-underline-at-descent-line",
	       x_underline_at_descent_line,
	       doc: /* Non-nil means to draw the underline at the same place as the descent line.
		       (If `line-spacing' is in effect, that moves the underline lower by
		       that many pixels.)
		       A value of nil means to draw the underline according to the value of the
		       variable `x-use-underline-position-properties', which is usually at the
		       baseline level.  The default value is nil.  */);
  x_underline_at_descent_line = false;
  DEFSYM (Qx_underline_at_descent_line, "x-underline-at-descent-line");

  DEFVAR_LISP ("x-select-enable-clipboard-manager",
	       Vx_select_enable_clipboard_manager,
	       doc: /* Whether to enable X clipboard manager support.
		       If non-nil, then whenever Emacs is killed or an Emacs frame is deleted
		       while owning the X clipboard, the clipboard contents are saved to the
		       clipboard manager if one is present.  */);
  Vx_select_enable_clipboard_manager = Qt;

#ifdef WEBRENDER_HEAD_REV
  DEFVAR_LISP ("webrender-head-rev", Vwebrender_head_rev,
	       doc: /* String containing the webrender head rev Emacs was built with.  */);
  Vwebrender_head_rev = build_string (WEBRENDER_HEAD_REV);
#endif

  syms_of_wrterm_rust();

  Fprovide (Qwr, Qnil);
}
