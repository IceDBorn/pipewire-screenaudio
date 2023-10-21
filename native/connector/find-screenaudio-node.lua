#!/usr/bin/env /usr/bin/wpexec
utils = require("utils"):Configure(Constraint, Log)

node_mgr = ObjectManager {
  Interest {
    type = "node",
    Constraint { "node.name", "=", "pipewire-screenaudio" },
  },
}

node_mgr:connect(
  "installed",
  function(om)
    local node = om:lookup()
    print(node.properties["object.id"])
    Core.quit()
  end
)

node_mgr:activate()
