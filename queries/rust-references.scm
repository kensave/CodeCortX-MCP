; Function calls
(call_expression
  function: (identifier) @reference.name) @reference.usage

(call_expression
  function: (scoped_identifier
    name: (identifier) @reference.name)) @reference.usage

; Field access
(field_expression
  field: (field_identifier) @reference.name) @reference.usage

; Variable/identifier references
(identifier) @reference.name

; Type references
(type_identifier) @reference.name

; Macro invocations
(macro_invocation
  macro: (identifier) @reference.name) @reference.usage

; Use declarations
(use_declaration
  argument: (identifier) @reference.import)
