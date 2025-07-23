FROM node:22@sha256:e515259afd26f60db74957c62203c93d45760f2ba864d94accfa2edfc1ac17cf
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
