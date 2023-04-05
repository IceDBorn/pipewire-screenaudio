#include <iostream>
#include <optional>
#include <rohrkabel/main_loop.hpp>
#include <rohrkabel/registry/registry.hpp>

std::map<std::uint32_t, pipewire::port> ports;
std::optional<pipewire::port> virt_fl, virt_fr;

std::map<std::uint32_t, pipewire::node> nodes;
std::map<std::uint32_t, pipewire::link> links;

void link(const std::string &target, pipewire::core &core)
{
    for (const auto &[port_id, port] : ports)
    {
        if (!virt_fl || !virt_fr)
            continue;

        if (links.count(port_id))
            continue;

        if (port.info().direction == pipewire::port_direction::input)
            continue;

        if (!port.info().props.count("node.id"))
            continue;

        auto parent_id = std::stoul(port.info().props["node.id"]);

        if (!nodes.count(parent_id))
            continue;

        auto &parent = nodes.at(parent_id);

        if (parent.info().props["object.serial"].find(target) != std::string::npos)
        {
            std::cout << "Link   : " << target << ":" << port_id << " -> ";

            if (port.info().props["audio.channel"] == "FL")
            {
                links.emplace(port_id, *core.create_simple<pipewire::link>(virt_fl->info().id, port_id).get());
                std::cout << virt_fl->info().id << std::endl;
            }
            else
            {
                links.emplace(port_id, *core.create_simple<pipewire::link>(virt_fr->info().id, port_id).get());
                std::cout << virt_fr->info().id << std::endl;
            }
        }
    }
}

//? Due to pipewire listing some ports before adding the node
//? we need to call link everytime a node or port is added to catch
//? un-linked ports which we will then link.

int main()
{
    auto main_loop = pipewire::main_loop();
    auto context = pipewire::context(main_loop);
    auto core = pipewire::core(context);
    auto reg = pipewire::registry(core);

    std::string target;
    std::cout << "Enter an application you'd like to link to the virtual microphone: ";

    std::cin >> target;
    std::cout << std::endl;

    auto virtual_mic = core.create("adapter",
                                   {
                                       {"node.name", "pipewire-screenaudio"},            //
                                       {"media.class", "Audio/Source/Virtual"},     //
                                       {"factory.name", "support.null-audio-sink"}, //
                                       {"audio.channels", "2"},                     //
                                       {"audio.position", "FL,FR"}                  //
                                   },
                                   pipewire::node::type, pipewire::node::version, pipewire::update_strategy::none)
                           .share();

    auto reg_events = reg.listen<pipewire::registry_listener>();
    reg_events.on<pipewire::registry_event::global>([&](const pipewire::global &global) {
        if (global.type == pipewire::node::type)
        {
            auto node = reg.bind<pipewire::node>(global.id).get();
            std::cout << "Added  : " << node->info().props["node.name"] << std::endl;

            if (!nodes.count(global.id))
            {
                nodes.emplace(global.id, std::move(*node));
                link(target, core);
            }
        }
        if (global.type == pipewire::port::type)
        {
            auto port = reg.bind<pipewire::port>(global.id).get();
            auto info = port->info();

            if (info.props.count("node.id"))
            {
                auto node_id = std::stoul(info.props["node.id"]);

                if (node_id == virtual_mic.get()->id() && info.direction == pipewire::port_direction::input)
                {
                    if (info.props["audio.channel"] == "FL")
                    {
                        virt_fl.emplace(std::move(*port));
                    }
                    else
                    {
                        virt_fr.emplace(std::move(*port));
                    }
                }
                else
                {
                    ports.emplace(global.id, std::move(*port));
                }

                link(target, core);
            }
        }
    });

    reg_events.on<pipewire::registry_event::global_removed>([&](const std::uint32_t id) {
        if (nodes.count(id))
        {
            auto info = nodes.at(id).info();
            std::cout << "Removed: " << info.props["node.name"] << std::endl;
            nodes.erase(id);
        }
        if (ports.count(id))
        {
            ports.erase(id);
        }
        if (links.count(id))
        {
            links.erase(id);
        }
    });

    while (true)
    {
        main_loop.run();
    }
    return 0;
}
