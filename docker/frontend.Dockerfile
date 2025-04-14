# Create a build of the project
FROM node:latest AS build
WORKDIR /build
COPY . .

RUN npm install
RUN npm run build

# Copy the build artifacts
FROM node:latest
WORKDIR /app
COPY --from=build /build .

# Run the app
ENTRYPOINT ["npm", "run", "start"]
