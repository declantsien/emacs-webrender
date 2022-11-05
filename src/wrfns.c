/* #include <config.h> */
/* #include "lisp.h" */

/* /\* DEFUN ("x-hide-tip", Fx_hide_tip, Sx_hide_tip, 0, 0, 0, *\/ */
/* /\*        doc: /\\* Hide the current tooltip window, if there is any. *\/ */
/* /\* Value is t if tooltip was open, nil otherwise.  *\\/) *\/ */
/* /\*   (void) *\/ */
/* /\* { *\/ */
/* /\*   return false; *\/ */
/* /\* } *\/ */

/* /\* DEFUN ("xw-display-color-p", Fxw_display_color_p, Sxw_display_color_p, 0, 1, 0, *\/ */
/* /\*        doc: /\\* Internal function called by `display-color-p', which see.  *\\/) *\/ */
/* /\*   (Lisp_Object terminal) *\/ */
/* /\* { *\/ */
/* /\*   /\\* check_pgtk_display_info (terminal); *\\/ *\/ */
/* /\*   return Qt; *\/ */
/* /\* } *\/ */

/* /\* DEFUN ("x-display-grayscale-p", Fx_display_grayscale_p, Sx_display_grayscale_p, 0, 1, 0, *\/ */
/* /\*        doc: /\\* Return t if the display supports shades of gray. *\/ */
/* /\* Note that color displays do support shades of gray. *\/ */
/* /\* The optional argument TERMINAL specifies which display to ask about. *\/ */
/* /\* TERMINAL should be a terminal object, a frame or a display name (a string). *\/ */
/* /\* If omitted or nil, that stands for the selected frame's display.  *\\/) *\/ */
/* /\*   (Lisp_Object terminal) *\/ */
/* /\* { *\/ */
/* /\*   return Qt; *\/ */
/* /\* } *\/ */

/* DEFUN ("wr-create-frame", Fwr_create_frame, Swr_create_frame, 1, 1, 0, */
/*        doc: /\* Make a new X window, which is called a "frame" in Emacs terms. */
/* Return an Emacs frame object.  PARMS is an alist of frame parameters. */
/* If the parameters specify that the frame should not have a minibuffer, */
/* and do not specify a specific minibuffer window to use, then */
/* `default-minibuffer-frame' must be a frame whose minibuffer can be */
/* shared by the new frame. */

/* This function is an internal primitive--use `make-frame' instead.  *\/ ) */
/*   (Lisp_Object parms) */
/* { */
/*   return Qnil; */
/* } */

/* void */
/* syms_of_wrfns (void) */
/* { */
/*   defsubr (&Swr_create_frame); */
/* } */
