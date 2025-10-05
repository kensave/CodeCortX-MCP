; Function calls
(call_expression
  (navigation_expression
    (identifier) @reference.name)) @reference.usage

(call_expression
  (identifier) @reference.name) @reference.usage

; Property/variable references
(identifier) @reference.name
