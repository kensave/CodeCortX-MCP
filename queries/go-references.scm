; Function calls
(call_expression
  function: (identifier) @reference.name) @reference.usage

; Method calls
(call_expression
  function: (selector_expression
    field: (field_identifier) @reference.name)) @reference.usage

; Field access
(selector_expression
  field: (field_identifier) @reference.name) @reference.usage

; Variable/identifier references
(identifier) @reference.name

; Type references
(type_identifier) @reference.name

; Package references
(qualified_type
  package: (package_identifier) @reference.name)

; Import statements
(import_spec
  path: (interpreted_string_literal) @reference.import)
