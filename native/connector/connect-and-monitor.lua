#!/usr/bin/wpexec


--local argv = ...

--print("Command-line arguments:")
--for k, v in pairs(argv) do
  --print("\t" .. k .. ": " .. v)
--end

node_mgr = ObjectManager {
  Interest {
    type = "node",
    Constraint { "media.name", "!", "AudioCallbackDriver" },
    --Constraint { "media.name", "!", "AudioCallbackDriver" },
    Constraint { "media.class", "=", "Stream/Output/Audio" },
  },
}
port_mgr = ObjectManager {
  Interest {
    type = "port",
  },
}

node_mgr:connect(
  "object-added",
  function(_, node)
    local node_id = node.properties["object.id"]
    local ports = {}
    for port in port_mgr:iterate {
      type = "port",
      Constraint { "node.id", "=", tostring(node_id) },
      Constraint { "audio.channel", "in-list", "FR", "FL" }
    } do
      local channel = port.properties["audio.channel"]
      local port_id = port.properties["object.id"]
      ports[channel] = port_id
    end
    print(ports["FL"] .. ' ' .. ports["FR"])
  end
)

port_mgr:activate()
node_mgr:activate()
