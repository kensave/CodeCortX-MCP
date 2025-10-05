; C Reference Tracking Queries

; Function calls
(call_expression
  function: (identifier) @reference.name) @reference.usage

; Variable references
(identifier) @reference.name

; Field access
(field_expression
  field: (field_identifier) @reference.name) @reference.usage

; Type references
(type_identifier) @reference.name
