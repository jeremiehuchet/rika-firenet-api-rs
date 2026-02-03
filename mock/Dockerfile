FROM node:22@sha256:379c51ac7bbf9bffe16769cfda3eb027d59d9c66ac314383da3fcf71b46d026c
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
