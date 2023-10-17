#!/usr/bin/wpexec

node_mgr = ObjectManager {
  Interest {
    type = "node",
    Constraint { "media.class", "=", "Stream/Output/Audio" },
    -- Repeat this line to add more exclusions
    Constraint { "media.name", "!", "AudioCallbackDriver", type = "pw" },
  },
}

-- prints FL and FR port ids from a node
function PrintPorts(node)
  local ports = {}
  for port in node:iterate_ports {
    Constraint { "audio.channel", "in-list", "FR", "FL" }
  } do
    local channel = port.properties["audio.channel"]
    local port_id = port.properties["object.id"]
    ports[channel] = port_id
  end
  if ports["FL"] == nil or ports["FR"] == nil then
    return
  end
  Log.info(node.properties["media.name"])
  print(ports["FL"] .. ' ' .. ports["FR"])
end

node_mgr:connect(
  "object-added",
  function(_, node)
    Log.info(node.properties["media.class"])
    PrintPorts(node)
    -- sometimes ports are added after node creation
    node:connect("ports-changed", function(node)
      PrintPorts(node)
    end)
  end
)

node_mgr:activate()
