; Method calls
(method_invocation
  name: (identifier) @reference.name) @reference.usage

; Field access
(field_access
  field: (identifier) @reference.name) @reference.usage

; Variable/identifier references
(identifier) @reference.name

; Type references
(type_identifier) @reference.name

; Constructor calls
(object_creation_expression
  (identifier) @reference.name) @reference.usage

; Import statements
(import_declaration
  (identifier) @reference.import)
