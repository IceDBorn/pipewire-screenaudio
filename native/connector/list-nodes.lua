#!/usr/bin/env /usr/bin/wpexec
utils = require("utils"):Configure(Constraint, Log)

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
    for node in node_mgr:iterate() do
      local props = {}
      for key, value in pairs(node.properties) do
        props[key] = value
      end
      table.insert(nodes, Json.Object {
        properties = Json.Object(props)
      })
    end
    print(Json.Array(nodes):get_data())
    Core.quit()
  end
)

node_mgr:activate()
