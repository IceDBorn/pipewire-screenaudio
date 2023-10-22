#!/usr/bin/env /usr/bin/wpexec
utils = require("utils"):Configure(Constraint, Log)

argv = ...

targetSerial = argv.targetSerial

node_finder = ObjectManager {
  Interest {
    type = "node",
    Constraint { "media.class", "=", "Stream/Output/Audio" },
    Constraint { "object.serial", "=", targetSerial, type = "pw" },
  },
}

node_finder:connect("installed", function()
  node = node_finder:lookup()

  if node ~= nil then
    utils:PrintPorts(node)
  end

  Core.quit()
end)

node_finder:activate()
