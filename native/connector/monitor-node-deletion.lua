#!/usr/bin/env /usr/bin/wpexec
utils = require("utils"):Configure(Constraint, Log, Interest)

argv = ...

targetId = argv.targetId

if targetId == nil then
  Core.quit()
  return
end

node_mgr = ObjectManager {
  Interest {
    type = "node",
    Constraint { "object.id", "=", targetId, type = "pw" },
  },
}

node_mgr:connect(
  "object-removed",
  function(_, node)
    Core.quit()
  end
)

node_mgr:activate()
