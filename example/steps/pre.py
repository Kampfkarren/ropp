# This will rename the Workspace to Ropp
# This is obviously not practical, but it's an easy example without needing an XML library to show it off.
import os

rbxmx_path = os.environ["ROPP_RBXMX"]

with open(rbxmx_path, "r") as rbxmx:
	content = rbxmx.read()

with open(rbxmx_path, "w") as rbxmx:
	rbxmx.write(content.replace(
		'<string name="Name">Workspace</string>',
		'<string name="Name">Ropp</string>'
	))
