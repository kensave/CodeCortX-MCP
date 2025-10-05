; Function calls
(call_expression
  function: (identifier) @reference.name) @reference.usage

(call_expression
  function: (field_expression
    field: (identifier) @reference.name)) @reference.usage

; Variable/value references
(identifier) @reference.name

; Type references
(type_identifier) @reference.name

; Import statements
(import_declaration
  (identifier) @reference.import)

; Field access
(field_expression
  field: (identifier) @reference.name) @reference.usage
