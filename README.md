[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/6FRwiRqU)
Goal: Apply the knowledge you've learned in new ways.

# Project description
This is an open-ended project. Students can extend their BearTV project or do something new from the ground up. Project ideas must be approved by Dr. Freeman.

You must give a **formal presentation** of your project in place of a final exam. Each group will have ~12 minutes to present their work. Each member of the group must speak. You should have slides. Your presentation must include a demo of your project, although it may invlude a pre-recorded screen capture. In your presentation, you should introduce the problem that you addressed, how you addressed it, technical challenges you faced, what you learned, and next steps (if you were to continue developing it).

You may use AI LLM tools to assist with the development of your project, including code assistant tools like GitHub Copilot. If you do use any AI tools, you must describe your use during your presentation.

Unless you get specific approval otherwise, your project **must** include some component deployed on a cloud hosting service. You can use AWS, GCP, Azure, etc. These services have free tiers, and you might consider looking into tiers specifically for students.

## Milestones
- You must meet with Dr. Freeman within the first week to get your project idea approved
- You must meet with Dr. Freeman within the first 3 weeks to give a status update and discuss roadblocks
- See the course schedule spreadhseet for specific dates

## Project Ideas
- Simulate UDP packet loss and packet corruption in BearTV in a non-deterministic way (i.e., don't just drop every Nth packet). Then, extend the application protocol to be able to detect and handle this packet loss.
- Extend the BearTV protocol to support streaming images (or video!) alongside the CC data, and visually display them on the client. This should be done in such a way that it is safely deliver*able* over *any* implementation of IPv4. The images don't have to be relevant to the caption data--you can get them randomly on the server from some image source.
- Do something hands on with a video streaming protocol such as MoQ, DASH, or HLS.
- Implement QUIC
- Develop a new congestion control algorithm and evaluate it compared to existing algorithms in a realistic setting
- Make significant contributions to a relevant open-source repository (e.g., moq-rs)
- Implement a VPN
- Implement a DNS
- Do something with route optimization
- Implement an HTTP protocol and have a simple website demo

--> These are just examples. I hope that you'll come up with a better idea to suit your own interests!

## Libraries

Depending on the project, there may be helpful libraries you find to help you out. However, there may also be libraries that do all the interesting work for you. Depending on the project, you'll need to determine what should be fair game. For example, if your project is to implement HTTP, then you shouldn't leverage an HTTP library that does it for you.

If you're unsure if a library is okay to use, just ask me.

## Languages

The core of your project should, ideally, be written in Rust. Depending on the project idea, however, I'm open to allowing the use of other languages if there's a good reason for it. For me to approve such a request, the use of a different language should enable greater learning opportunities for your group.

# Submission

## Questions
- What is your project?
	- Our project is a chat application utilizing QUIC. It takes advantage of QUIC's efficiency in creating concurrent bidirectional streams to send various responses and commands back and forth between the client and the server, hosted on GCP. A user can use the application to chat with friends through the terminal, using a UI created with the ratatui crate.
- What novel work did you do?
	- Utilizing a QUIC server.
	- Creating bidirectional streams that only exist for a single RPC command/response pair.
	- Creating a terminal UI.
	- Generating self-signed certificates.
- What did you learn?
	- QUIC is more useful than TCP for creating and dropping streams quickly for the sake of single command communication between two endpoints.
- What was challenging?
	- Learning which versions of the crates would work for our purposes. Not only did we have to consider how the crates might react to each other, but also if they could be used universally on any machine and if they could be utilized on our GCP host.
	- Learning how to effectively utilize the advantages of QUIC. Trying to utilize it in the same way as TCP was not effective and eventually learning that it was actually more effective to create and drop individual streams to take advantage of QUIC's concurrency handling took some experimentation and online research.
- What AI tools did you use, and what did you use them for? What were their benefits and drawbacks?
	- ChatGPT was used for debugging and research.
		- Benefits: ChatGPT was great at identifying errors that we had not seen before, such as the many crate issues. It was also useful in quickly generating useful debug lines that could be used to solve an immediate issue and then be immediately removed.
		- Drawbacks: ChatGPT, while helpful at identifying why the crates were causing errors, was not so helpful at determining what version of each crate would work correctly and what syntax had changed between each version.
- What would you do differently next time?
	- Look at the documentation for Quinn and Rustls to quickly learn of any common issues that might be encountered before starting to code.
	- Do some pre-programming research on what crates might be usefull rather than finding the crates as they are needed. This would prevent having to divert from the current task to solve crate conflicts.

## What to submit
- Push your working code to the main branch of your team's GitHub Repository before the deadline
- Edit the README to answer the above questions
- On Teams, *each* member of the group must individually upload answers to these questions:
	- What did you (as an individual) contribute to this project?
	- What did the other members of your team contribute?
	- Do you have any concerns about your own performance or that of your team members? Any comments will remain confidential, and Dr. Freeman will try to address them in a way that preserves anonymity.
	- What feedback do you have about this course?

## Grading

Grading will be based on...
- The technical merit of the group's project
- The contribution of each individual group member
- Evidence of consistent work, as revealed during milestone meetings
- The quality of the final presentation