; TypeScript/TSX Reference Tracking Queries

; Function calls
(call_expression
  function: (identifier) @reference.name) @reference.usage

(call_expression
  function: (member_expression
    property: (property_identifier) @reference.name)) @reference.usage

; Variable/identifier references
(identifier) @reference.name

; Member access
(member_expression
  property: (property_identifier) @reference.name) @reference.usage

; Type references
(type_identifier) @reference.name

; Import references
(import_specifier
  name: (_) @reference.name) @reference.usage


