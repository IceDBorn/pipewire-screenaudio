#!/usr/bin/env wpexec
utils = require("utils"):Configure(Constraint, Log, Interest)

argv = ...

targetSerial = argv.targetSerial
direction = argv.direction

if targetSerial == nil or direction == nil then
  Core.quit()
  return
end

node_finder = ObjectManager {
  Interest {
    type = "node",
    Constraint { "media.class", "=", "Stream/Output/Audio" },
    Constraint { "object.serial", "=", targetSerial, type = "pw" },
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
