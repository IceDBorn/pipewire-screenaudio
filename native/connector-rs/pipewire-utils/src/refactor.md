Write an abstraction to hide all the pipewire boilerplate from the main code


It should allow me to:

- iterate over full node data using proxied objects:
	- we need multiple properties available in the proxies
- iterate over basic port data:
	- to find who they are connected to

- monitor new nodes with their new ports:
	- we need some node properties to filter on
	- we need the port IDs to connect them with the virtmic
	- we need to be able to interrupt it with a signal



