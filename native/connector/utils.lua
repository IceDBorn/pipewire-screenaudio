local utils = {}

function utils:Configure(Constraint, Log)
  self.Constraint = Constraint
  self.Log = Log
  return self
end

-- prints FL and FR port ids from a node
function utils:PrintPorts(node)
  local ports = {}
  for port in node:iterate_ports {
    self.Constraint { "audio.channel", "in-list", "FR", "FL" }
  } do
    local channel = port.properties["audio.channel"]
    local port_id = port.properties["object.id"]
    ports[channel] = port_id
  end
  if ports["FL"] == nil or ports["FR"] == nil then
    return
  end
  self.Log.info(node.properties["media.name"])
  print(ports["FL"] .. ' ' .. ports["FR"])
end


return utils
