# Create a build of the project
FROM node:latest AS build
WORKDIR /build
COPY . .

RUN npm install
RUN npm run build

# Copy the build artifacts
FROM node:latest
WORKDIR /app
COPY --from=build /build/.next ./.next
COPY --from=build /build/package.json ./package.json
COPY --from=build /build/package-lock.json ./package-lock.json
RUN npm install --only=production

# Run the app
ENTRYPOINT ["npm", "run", "start"]
