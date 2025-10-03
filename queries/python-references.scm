; Function calls
(call
  function: (identifier) @reference.name) @reference.usage

(call
  function: (attribute
    attribute: (identifier) @reference.name)) @reference.usage

; Variable/identifier references
(identifier) @reference.name

; Attribute access
(attribute
  attribute: (identifier) @reference.name) @reference.usage

; Import statements
(import_statement
  name: (dotted_name) @reference.import)

(import_from_statement
  name: (dotted_name) @reference.import)
