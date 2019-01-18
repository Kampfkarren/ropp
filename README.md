<div align="center">
	<h3>Ropp</h3>
	<b>Ro</b>blox <b>P</b>re/Post <b>P</b>ublishing
</div>

----------

<b>Problem: </b> You want to change your place/code after you publish, but don't want to make it harder to develop by directly editing.

<b>Solution: </b> Publish through Ropp, rather than through Roblox.

<h3>How it works</h3>

 - Develop your place, preferably on its own "developer build".
 - When you're done, save the rbxmx.
 - Write a ropp.json file to list all your steps.
 - Run ropp, give it your .ROBLOSECURITY to log in, and have Ropp publish it for you!

<h3>Use cases</h3>

- Variable obfuscation
- RemoteEvent obfuscation
- Automatic changelog creation

<h3>Q/A</h3>

Q: What if Ropp somehow messes up my place when publishing?
A: Roblox has built in previous versions, revert to one of those, and make an issue about it. Make sure it's Ropp and not one of your publish steps.
