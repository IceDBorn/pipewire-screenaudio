#!/usr/bin/env wpexec
utils = require("utils"):Configure(Constraint, Log, Interest)

node_mgr = ObjectManager {
  Interest {
    type = "node",
    Constraint { "media.class", "=", "Stream/Output/Audio" },
  },
}

node_mgr:connect(
  "installed",
  function(om)
    local nodes = {
      Json.Object {
        properties = Json.Object {
          ["media.name"] = "[All Desktop Audio]",
          ["application.name"] = "",
          ["object.serial"] = -1
        }
      }
    }
    for node in om:iterate() do
      local props = node.properties
      table.insert(nodes, Json.Object {
        properties = Json.Object {
          ["media.name"] = props["media.name"],
          ["application.name"] = props["application.name"],
          ["object.serial"] = tonumber(props["object.serial"]),
        }
      })
    end
    print(Json.Array(nodes):get_data())
    Core.quit()
  end
)

node_mgr:activate()
