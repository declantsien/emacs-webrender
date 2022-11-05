;;; Emacs NG - A new approach to Emacs.

;;; Commentary:
;; To use as the basis for a development environment, run:
;;
;;   guix shell
;;
;; GNU Guix development package.  To build, run:
;;
;;   guix build -f guix.scm
;;
;;; Code:

(use-modules (guix gexp)
	     (guix utils)
	     (guix git)
	     (guix packages)
	     (guix download)
	     (guix git-download)
	     ((guix licenses) #:prefix license:)
	     (guix build-system meson)
	     (guix build-system cargo)
	     (gnu packages crates-io)
	     (gnu packages emacs)
	     (gnu packages llvm)
	     (gnu packages gcc)
	     (gnu packages gl)
	     (binary packages rust)
	     (gnu packages fontutils)
	     (gnu packages gtk)
	     (gnu packages freedesktop)
	     (gnu packages xdisorg)
	     (gnu packages gdb)
	     (gnu packages base)
	     (gnu packages valgrind)
	     (gnu packages linux)
	     (gnu packages python)
	     (gnu packages python-build)
	     (gnu packages commencement)
	     (gnu packages shellutils)
	     (gnu packages))

;; (define-public rust-bindgen
;;   (package
;;     (name "rust-bindgen")
;;     (version "0.61.0")
;;     (source (origin
;;               (method url-fetch)
;;               (uri (crate-uri "bindgen" version))
;;               (file-name (string-append name "-" version ".tar.gz"))
;;               (sha256
;;                (base32
;;                 "16phlka8ykx28jlk7l637vlr9h01j8mh2s0d6km6z922l5c2w0la"))))
;;     (build-system cargo-build-system)
;;     (arguments
;;      `(#:tests? #f
;;        #:cargo-inputs (("rust-bitflags" ,rust-bitflags-1)
;;                        ("rust-cexpr" ,rust-cexpr-0.6)
;;                        ("rust-clang-sys" ,rust-clang-sys-1)
;;                        ("rust-lazy-static" ,rust-lazy-static-1)
;;                        ("rust-lazycell" ,rust-lazycell-1)
;;                        ("rust-log" ,rust-log-0.4)
;;                        ("rust-peeking-take-while" ,rust-peeking-take-while-0.1)
;;                        ("rust-proc-macro2" ,rust-proc-macro2-1)
;;                        ("rust-quote" ,rust-quote-1)
;;                        ("rust-regex" ,rust-regex-1)
;;                        ("rust-rustc-hash" ,rust-rustc-hash-1)
;;                        ("rust-shlex" ,rust-shlex-1)
;;                        ("rust-syn" ,rust-syn-1)
;;                        ("rust-which" ,rust-which-4))))
;;     (home-page "https://rust-lang.github.io/rust-bindgen/")
;;     (synopsis
;;      "Automatically generates Rust FFI bindings to C and C++ libraries.")
;;     (description
;;      "Automatically generates Rust FFI bindings to C and C++ libraries.")
;;     (license license:bsd-3)))

(define-public rust-bindgen
  (package
    (inherit rust-bindgen-0.46)
    (arguments
     (substitute-keyword-arguments (package-arguments rust-bindgen-0.59)
       ((#:skip-build? _ #t) #f)
       ((#:tests? _ #t) #f)
       ))))

(package
  (inherit emacs)
  (name "emacs-ng")
  (version "0.1")
  (source (git-checkout (url (dirname (current-filename)))))
  (build-system meson-build-system)
  (home-page "https://github.com/emacs-ng/emacs-ng")
  (synopsis "A new approach to Emacs")
  (native-inputs
     (modify-inputs (package-native-inputs emacs)
       (append rust-nightly-2022-10-24-x86_64-linux
	       `(,rust-nightly-2022-10-24-x86_64-linux "cargo")
	       `(,rust-nightly-2022-10-24-x86_64-linux "rust-docs")
               `(,rust-nightly-2022-10-24-x86_64-linux "rust-docs-json-preview")
               `(,rust-nightly-2022-10-24-x86_64-linux "clippy-preview")
               `(,rust-nightly-2022-10-24-x86_64-linux "rust-analyzer-preview")
               ;; `(,rust-nightly-2022-10-24-x86_64-linux "rust-demangler-preview")
               `(,rust-nightly-2022-10-24-x86_64-linux "rustfmt-preview")
	       clang-toolchain
	       ;; gcc-toolchain
	       gdb
	       valgrind
	       strace
	       `(,glibc "debug")
	       ;; gcc
	       ;; `(,gcc "lib")
	       mesa
	       freetype
	       harfbuzz
	       wayland
	       mesa
	       libxkbcommon
	       python
	       python-pytoml)))
  (arguments
   (list ))
  (description
   "A new approach to Emacs - Including TypeScript, Threading,
Async I/O, and WebRender.")
  (license license:gpl3+))
