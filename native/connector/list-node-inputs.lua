#!/usr/bin/env /usr/bin/wpexec
utils = require("utils"):Configure(Constraint, Log, Interest)

argv = ...

nodeId = argv.nodeId

if nodeId == nil then
  Core.quit()
  return
end

link_mgr = ObjectManager {
  Interest {
    type = "link",
    Constraint { "link.input.node", "=", nodeId, type = "pw" },
  },
}

link_mgr:connect(
  "installed",
  function(om)
    for edge in om:iterate() do
      print(edge.properties["object.id"])
    end
    Core.quit()
  end
)

link_mgr:activate()
