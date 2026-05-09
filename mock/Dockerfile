FROM node:22@sha256:62e4daa6819762bbd3072af77cc282ab72c631c4aed30dd7980192babaf385b3
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
