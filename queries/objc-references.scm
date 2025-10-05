; Objective-C Reference Extraction Queries

; Function calls
(call_expression
  function: (identifier) @reference.name) @reference.usage

; Message expressions
(message_expression
  (identifier) @reference.name) @reference.usage

; Variable references
(identifier) @reference.name

; Type references
(type_identifier) @reference.name
