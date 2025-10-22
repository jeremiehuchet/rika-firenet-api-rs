FROM node:22@sha256:a2a7dccd1160275ea1aa1b4dddd1f2886f2092d502877eb84682d2b88c3915c4
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
