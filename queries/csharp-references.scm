; Method calls
(invocation_expression
  function: (member_access_expression
    name: (identifier) @reference.name)) @reference.usage

; Simple method calls
(invocation_expression
  function: (identifier) @reference.name) @reference.usage

; Variable references
(identifier) @reference.name

; Member access
(member_access_expression
  name: (identifier) @reference.name) @reference.usage

; Type references in object creation
(object_creation_expression
  type: (identifier) @reference.name) @reference.usage
