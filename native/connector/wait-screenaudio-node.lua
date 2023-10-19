#!/usr/bin/env wpexec
utils = require("utils"):Configure(Constraint, Log, Interest)

function PrintNodeID(node)
  print(node.properties["object.id"])
end

node_mgr = ObjectManager {
  utils.screenaudioNode
}

node_mgr:connect(
  "object-added",
  function(om, node)
    PrintNodeID(node)
    Core.quit()
  end
)

node_mgr:activate()
