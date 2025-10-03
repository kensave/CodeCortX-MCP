; Methods
(method
  name: (identifier) @method.name
) @method.definition

; Classes
(class
  name: (constant) @class.name
) @class.definition

; Modules
(module
  name: (constant) @module.name
) @module.definition

; Constants
(assignment
  left: (constant) @const.name
) @const.definition
