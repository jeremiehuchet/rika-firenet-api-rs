FROM node:20@sha256:940071b4e4c9918c40c632a4b51861aded3087696a2847a5d8ecff3546eb13ce
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
