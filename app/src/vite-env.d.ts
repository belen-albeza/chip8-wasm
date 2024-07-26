/// <reference types="vite/client" />
declare module "*.svg" {
  const content: SVGElement;
  export default content;
}

declare module "*.ch8" {
  const content: string;
  export default content;
}
