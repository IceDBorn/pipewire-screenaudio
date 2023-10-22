#!/usr/bin/env /usr/bin/wpexec
utils = require("utils"):Configure(Constraint, Log)

node_mgr = ObjectManager {
  Interest {
    type = "node",
    Constraint { "media.class", "=", "Stream/Output/Audio" },
    -- Repeat this line to add more exclusions
    Constraint { "media.name", "!", "AudioCallbackDriver", type = "pw" },
  },
}

node_mgr:connect(
  "object-added",
  function(_, node)
    local props = node.properties;
    Log.info(props["media.name"] .. ' [' .. props["object.serial"] .. ']')
    utils:PrintPorts(node)
    -- sometimes ports are added after node creation
    node:connect("ports-changed", function(node)
      utils:PrintPorts(node)
    end)
  end
)

node_mgr:activate()
