#!/usr/bin/env /usr/bin/wpexec
utils = require("utils"):Configure(Constraint, Log, Interest)

node_mgr = ObjectManager {
  utils.screenaudioNode
}

node_mgr:connect(
  "installed",
  function(om)
    local node = om:lookup()
    if node ~= nil then
      print(node.properties["object.id"])
    end
    Core.quit()
  end
)

node_mgr:activate()
