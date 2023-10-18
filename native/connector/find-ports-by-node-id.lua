#!/usr/bin/env /usr/bin/wpexec
utils = require("utils"):Configure(Constraint, Log, Interest)

argv = ...

targetId = argv.targetId
direction = argv.direction

if targetId == nil or direction == nil then
  Core.quit()
  return
end

node_finder = ObjectManager {
  Interest {
    type = "node",
    Constraint { "object.id", "=", targetId, type = "pw" },
  },
}

node_finder:connect("installed", function(om)
  node = om:lookup()

  if node ~= nil then
    utils:PrintPorts(node, direction)
  end

  Core.quit()
end)

node_finder:activate()
