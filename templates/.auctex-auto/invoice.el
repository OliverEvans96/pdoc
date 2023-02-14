(TeX-add-style-hook
 "invoice"
 (lambda ()
   (TeX-add-to-alist 'LaTeX-provided-class-options
                     '(("CSMinimalInvoice" "	letterpaper" "	10pt" "")))
   (TeX-run-style-hooks
    "latex2e"
    "CSMinimalInvoice"
    "CSMinimalInvoice10"))
 :latex)

